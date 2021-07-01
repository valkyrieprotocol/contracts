use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Uint128, CosmosMsg, WasmMsg, to_binary};
use valkyrie::common::ContractResult;
use crate::executions::remove_campaign;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_not_found_err};
use crate::tests::governance_sender;
use crate::tests::add_campaign::CAMPAIGN1;
use valkyrie::campaign::execute_msgs::ExecuteMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    campaign_address: String,
) -> ContractResult<Response> {
    remove_campaign(deps.as_mut(), env, info, campaign_address)
}

pub fn will_success(deps: &mut CustomDeps, campaign_address: String) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(deps, env.clone(), info.clone(), campaign_address).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let campaign = CAMPAIGN1.to_string();
    let spend_limit = Uint128(100);

    super::add_campaign::will_success(&mut deps, campaign.clone(), spend_limit);

    let (_, _, response) = will_success(&mut deps, campaign.clone());
    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: campaign,
            send: vec![],
            msg: to_binary(&ExecuteMsg::DeregisterBooster {}).unwrap(),
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
        CAMPAIGN1.to_string(),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_not_found() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        CAMPAIGN1.to_string(),
    );

    expect_not_found_err(&result);
}
