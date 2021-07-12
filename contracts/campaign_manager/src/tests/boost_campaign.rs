use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, CosmosMsg, WasmMsg, to_binary, Decimal};
use valkyrie::common::ContractResult;
use crate::executions::boost_campaign;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_generic_err};
use crate::tests::{governance_sender, MIN_PARTICIPATION_COUNT, FUND_MANAGER, DROP_BOOSTER_RATIO_PERCENT, ACTIVITY_BOOSTER_RATIO_PERCENT, PLUS_BOOSTER_RATIO_PERCENT, ACTIVITY_BOOSTER_MULTIPLIER_PERCENT};
use valkyrie::campaign::query_msgs::CampaignStateResponse;
use valkyrie::fund_manager::execute_msgs::ExecuteMsg as FundExecuteMsg;
use valkyrie::campaign::execute_msgs::ExecuteMsg as CampaignExecuteMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    campaign: String,
    amount: Uint128,
) -> ContractResult<Response> {
    boost_campaign(deps.as_mut(), env, info, campaign, amount)
}

pub fn will_success(
    deps: &mut CustomDeps,
    campaign: String,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        campaign,
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    let campaign = "Campaign";
    let amount = Uint128(1000);
    deps.querier.with_campaign_state(campaign.to_string(), CampaignStateResponse {
        participation_count: MIN_PARTICIPATION_COUNT,
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        balance: Uint128::zero(),
        is_active: true,
        is_pending: false,
    });

    super::instantiate::default(&mut deps);

    let (_, _, response) = will_success(
        &mut deps,
        campaign.to_string(),
        amount.clone(),
    );

    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: FUND_MANAGER.to_string(),
            send: vec![],
            msg: to_binary(&FundExecuteMsg::IncreaseAllowance {
                address: campaign.to_string(),
                amount: amount.clone(),
            }).unwrap(),
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: campaign.to_string(),
            send: vec![],
            msg: to_binary(&CampaignExecuteMsg::EnableBooster {
                drop_booster_amount: Decimal::percent(DROP_BOOSTER_RATIO_PERCENT) * amount,
                activity_booster_amount: Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT) * amount,
                plus_booster_amount: Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT) * amount,
                activity_booster_multiplier: Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
            }).unwrap(),
        }),
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
        Uint128(100),
    );
    expect_unauthorized_err(&result);
}

#[test]
fn failed_boost_criteria() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    deps.querier.with_campaign_state("Campaign1".to_string(), CampaignStateResponse {
        participation_count: MIN_PARTICIPATION_COUNT - 1,
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        balance: Uint128::zero(),
        is_active: true,
        is_pending: false,
    });

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        "Campaign1".to_string(),
        Uint128(1),
    );
    expect_generic_err(&result, "Not satisfied min_participation_count");

    deps.querier.with_campaign_state("Campaign2".to_string(), CampaignStateResponse {
        participation_count: MIN_PARTICIPATION_COUNT,
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        balance: Uint128::zero(),
        is_active: true,
        is_pending: true,
    });

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        "Campaign2".to_string(),
        Uint128(1),
    );
    expect_generic_err(&result, "Can not boost in pending state");
}
