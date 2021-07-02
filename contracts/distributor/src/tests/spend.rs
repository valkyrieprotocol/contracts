use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, CosmosMsg, WasmMsg, to_binary, Addr};
use valkyrie::common::ContractResult;
use crate::executions::spend;
use valkyrie::test_utils::{contract_env, DEFAULT_SENDER, default_sender, expect_exceed_limit_err, expect_unauthorized_err};
use cosmwasm_std::testing::mock_info;
use crate::tests::add_campaign::CAMPAIGN1;
use crate::tests::{TOKEN_CONTRACT, GOVERNANCE};
use cw20::Cw20ExecuteMsg;
use crate::states::CampaignInfo;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    spend(deps.as_mut(), env, info, recipient, amount)
}

pub fn will_success(
    deps: &mut CustomDeps,
    sender: &str,
    recipient: String,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(sender, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        recipient,
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed_campaign() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::add_campaign::will_success(
        &mut deps,
        CAMPAIGN1.to_string(),
        Uint128(100),
    );

    let (_, _, response) = will_success(
        &mut deps,
        CAMPAIGN1,
        DEFAULT_SENDER.to_string(),
        Uint128(1),
    );
    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: DEFAULT_SENDER.to_string(),
                amount: Uint128(1),
            }).unwrap(),
            send: vec![],
        }),
    ]);
}

#[test]
fn succeed_governance() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let (_, _, response) = will_success(
        &mut deps,
        GOVERNANCE,
        DEFAULT_SENDER.to_string(),
        Uint128(1),
    );
    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: DEFAULT_SENDER.to_string(),
                amount: Uint128(1),
            }).unwrap(),
            send: vec![],
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
        DEFAULT_SENDER.to_string(),
        Uint128(1),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_exceed_limit() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::add_campaign::will_success(
        &mut deps,
        CAMPAIGN1.to_string(),
        Uint128(100),
    );

    will_success(
        &mut deps,
        CAMPAIGN1,
        DEFAULT_SENDER.to_string(),
        Uint128(99),
    );

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(CAMPAIGN1, &[]),
        DEFAULT_SENDER.to_string(),
        Uint128(2),
    );

    expect_exceed_limit_err(&result);
}

#[test]
fn delete_after_exceed_limit() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::add_campaign::will_success(
        &mut deps,
        CAMPAIGN1.to_string(),
        Uint128(100),
    );

    will_success(
        &mut deps,
        CAMPAIGN1,
        DEFAULT_SENDER.to_string(),
        Uint128(100),
    );

    let campaign = CampaignInfo::may_load(
        &deps.storage,
        &Addr::unchecked(CAMPAIGN1),
    ).unwrap();
    assert!(campaign.is_none());
}
