use cosmwasm_std::{Addr, CosmosMsg, Decimal, Env, MessageInfo, Response, to_binary, Uint128, WasmMsg};

use valkyrie::campaign::execute_msgs::ExecuteMsg;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, default_sender, expect_already_exists_err, expect_unauthorized_err};

use crate::executions::add_campaign;
use crate::states::CampaignInfo;
use crate::tests::{ACTIVITY_BOOSTER_RATIO_PERCENT, DROP_BOOSTER_RATIO_PERCENT, governance_sender, PLUS_BOOSTER_RATIO_PERCENT};

pub const CAMPAIGN1: &str = "Campaign1";

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    campaign_address: String,
    spend_limit: Uint128,
) -> ContractResult<Response> {
    add_campaign(deps.as_mut(), env, info, campaign_address, spend_limit)
}

pub fn will_success(
    deps: &mut CustomDeps,
    campaign_address: String,
    spend_limit: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        campaign_address,
        spend_limit,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let spend_limit = Uint128(100);
    let drop_booster_amount = Decimal::percent(DROP_BOOSTER_RATIO_PERCENT) * spend_limit;
    let activity_booster_amount = Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT) * spend_limit;
    let plus_booster_amount = Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT) * spend_limit;

    let (_, _, response) = will_success(&mut deps, CAMPAIGN1.to_string(), spend_limit);
    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: CAMPAIGN1.to_string(),
            send: vec![],
            msg: to_binary(&ExecuteMsg::RegisterBooster {
                drop_booster_amount,
                activity_booster_amount,
                plus_booster_amount,
            }).unwrap(),
        }),
    ]);

    let campaign = CampaignInfo::load(
        &deps.storage,
        &Addr::unchecked(CAMPAIGN1),
    ).unwrap();
    assert_eq!(campaign, CampaignInfo {
        campaign: Addr::unchecked(CAMPAIGN1),
        spend_limit,
    });
}

#[test]
fn succeed_exist_finished_boost() {
    //TODO:
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        CAMPAIGN1.to_string(),
        Uint128(100),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_in_boosting() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    will_success(&mut deps, CAMPAIGN1.to_string(), Uint128(100));

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        CAMPAIGN1.to_string(),
        Uint128(100),
    );

    expect_already_exists_err(&result);
}