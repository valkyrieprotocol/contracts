use cosmwasm_std::{CosmosMsg, Env, MessageInfo, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{default_sender, DEFAULT_SENDER};
use valkyrie::test_constants::governance::{GOVERNANCE, governance_sender};
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::executions::transfer;
use valkyrie::test_constants::distributor::{distributor_env, MANAGING_TOKEN, DISTRIBUTOR, ADMINS};

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
    let env = distributor_env();
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
fn succeed() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(DISTRIBUTOR, &Uint128::new(10100))],
    )]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
        None,
    );

    let (_, _, response) = will_success(
        &mut deps,
        ADMINS[0],
        GOVERNANCE.to_string(),
        Uint128::new(100),
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MANAGING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: GOVERNANCE.to_string(),
                amount: Uint128::new(100),
            }).unwrap(),
            funds: vec![],
        })),
    ]);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        distributor_env(),
        default_sender(),
        DEFAULT_SENDER.to_string(),
        Uint128::new(1),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_insufficient_free_balance() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(DISTRIBUTOR, &Uint128::new(10100))],
    )]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
        None,
    );

    let result = exec(
        &mut deps,
        distributor_env(),
        governance_sender(),
        GOVERNANCE.to_string(),
        Uint128::new(101),
    );
    expect_generic_err(&result, "Insufficient balance");
}