use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Uint128, CosmosMsg, WasmMsg, to_binary, SubMsg};
use valkyrie::common::ContractResult;
use crate::executions::finish_boosting;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_generic_err};
use crate::tests::{governance_sender, FUND_MANAGER, MIN_PARTICIPATION_COUNT};
use valkyrie::fund_manager::execute_msgs::ExecuteMsg as FundExecuteMsg;
use valkyrie::campaign::execute_msgs::ExecuteMsg as CampaignExecuteMsg;
use valkyrie::campaign::query_msgs::{CampaignStateResponse, BoosterResponse, DropBoosterResponse, ActivityBoosterResponse, PlusBoosterResponse};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    campaign: String,
) -> ContractResult<Response> {
    finish_boosting(deps.as_mut(), env, info, campaign)
}

pub fn will_success(deps: &mut CustomDeps, campaign: String) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        campaign,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let campaign = "Campaign";
    deps.querier.with_campaign_state(campaign.to_string(), CampaignStateResponse {
        participation_count: MIN_PARTICIPATION_COUNT,
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        balance: Uint128::zero(),
        is_active: true,
        is_pending: false,
    });

    let (env, _, _) = super::boost_campaign::will_success(&mut deps, campaign.to_string(), Uint128::new(1));
    deps.querier.with_active_booster(campaign.to_string(), Some(BoosterResponse {
        drop_booster: DropBoosterResponse {
            assigned_amount: Uint128::from(100u64),
            calculated_amount: Uint128::zero(),
            reward_amounts: vec![],
            spent_amount: Uint128::zero(),
            snapped_participation_count: 0,
            snapped_distance_counts: vec![],
        },
        activity_booster: ActivityBoosterResponse {
            assigned_amount: Uint128::from(800u64),
            distributed_amount: Uint128::zero(),
            reward_amounts: vec![Uint128::from(500u64), Uint128::from(300u64), Uint128::from(200u64)],
        },
        plus_booster: PlusBoosterResponse {
            assigned_amount: Uint128::from(100u64),
            distributed_amount: Uint128::zero(),
        },
        boosted_at: env.block.time.clone(),
        finished_at: None,
    }));

    let (_, _, response) = will_success(&mut deps, campaign.to_string());

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: FUND_MANAGER.to_string(),
            funds: vec![],
            msg: to_binary(&FundExecuteMsg::DecreaseAllowance {
                address: campaign.to_string(),
                amount: Some(Uint128::from(1000u64)),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: campaign.to_string(),
            funds: vec![],
            msg: to_binary(&CampaignExecuteMsg::DisableBooster {}).unwrap(),
        })),
    ]);
}

#[test]
fn succeed_zero_release_amount() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let campaign = "Campaign";
    deps.querier.with_campaign_state(campaign.to_string(), CampaignStateResponse {
        participation_count: MIN_PARTICIPATION_COUNT,
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        balance: Uint128::zero(),
        is_active: true,
        is_pending: false,
    });

    let (env, _, _) = super::boost_campaign::will_success(&mut deps, campaign.to_string(), Uint128::new(1));
    deps.querier.with_active_booster(campaign.to_string(), Some(BoosterResponse {
        drop_booster: DropBoosterResponse {
            assigned_amount: Uint128::from(100u64),
            calculated_amount: Uint128::from(100u64),
            reward_amounts: vec![],
            spent_amount: Uint128::zero(),
            snapped_participation_count: 0,
            snapped_distance_counts: vec![],
        },
        activity_booster: ActivityBoosterResponse {
            assigned_amount: Uint128::from(800u64),
            distributed_amount: Uint128::from(800u64),
            reward_amounts: vec![Uint128::from(500u64), Uint128::from(300u64), Uint128::from(200u64)],
        },
        plus_booster: PlusBoosterResponse {
            assigned_amount: Uint128::from(100u64),
            distributed_amount: Uint128::from(100u64),
        },
        boosted_at: env.block.time.clone(),
        finished_at: None,
    }));

    let (_, _, response) = will_success(&mut deps, campaign.to_string());

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: campaign.to_string(),
            funds: vec![],
            msg: to_binary(&CampaignExecuteMsg::DisableBooster {}).unwrap(),
        })),
    ]);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        "Campaign".to_string(),
    );
    expect_unauthorized_err(&result);
}

#[test]
fn failed_non_active_booster() {
    let mut deps = custom_deps(&[]);
    deps.querier.with_active_booster(
        "Campaign".to_string(),
        None,
    );

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        "Campaign".to_string(),
    );
    expect_generic_err(&result, "Not exist active booster");
}
