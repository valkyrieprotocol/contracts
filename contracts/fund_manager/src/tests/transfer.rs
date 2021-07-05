use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, CosmosMsg, WasmMsg, to_binary, Addr};
use valkyrie::common::ContractResult;
use crate::executions::transfer;
use valkyrie::test_utils::{contract_env, DEFAULT_SENDER, default_sender, expect_unauthorized_err, expect_exceed_limit_err, expect_generic_err};
use cosmwasm_std::testing::{mock_info, MOCK_CONTRACT_ADDR};
use crate::tests::{ALLOWED_ADDRESS, TOKEN_CONTRACT, GOVERNANCE, governance_sender};
use cw20::Cw20ExecuteMsg;
use crate::states::Allowance;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    transfer(deps.as_mut(), env, info, recipient, amount)
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
fn succeed_allowed() {
    let mut deps = custom_deps(&[]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128(100),
    );

    let (_, _, response) = will_success(
        &mut deps,
        ALLOWED_ADDRESS,
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
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128(100))],
    )]);

    super::instantiate::default(&mut deps);

    let (_, _, response) = will_success(
        &mut deps,
        GOVERNANCE,
        DEFAULT_SENDER.to_string(),
        Uint128(100),
    );
    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: DEFAULT_SENDER.to_string(),
                amount: Uint128(100),
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
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128(100),
    );

    will_success(
        &mut deps,
        ALLOWED_ADDRESS,
        DEFAULT_SENDER.to_string(),
        Uint128(99),
    );

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(ALLOWED_ADDRESS, &[]),
        DEFAULT_SENDER.to_string(),
        Uint128(2),
    );

    expect_exceed_limit_err(&result);
}

#[test]
fn delete_after_exceed_limit() {
    let mut deps = custom_deps(&[]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128(100),
    );

    will_success(
        &mut deps,
        ALLOWED_ADDRESS,
        DEFAULT_SENDER.to_string(),
        Uint128(100),
    );

    let campaign = Allowance::may_load(
        &deps.storage,
        &Addr::unchecked(ALLOWED_ADDRESS),
    ).unwrap();
    assert!(campaign.is_none());
}

#[test]
fn failed_insufficient_free_balance() {
    let mut deps = custom_deps(&[]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128(100))],
    )]);

    super::instantiate::default(&mut deps);

    super::increase_allowance::will_success(
        &mut deps,
        ALLOWED_ADDRESS.to_string(),
        Uint128(100),
    );

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        DEFAULT_SENDER.to_string(),
        Uint128(1),
    );

    expect_generic_err(&result, "Insufficient balance");
}