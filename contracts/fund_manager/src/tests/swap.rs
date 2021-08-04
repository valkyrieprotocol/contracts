use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, coin, CosmosMsg, WasmMsg, to_binary, Addr, SubMsg};
use valkyrie::common::{ContractResult, Denom};
use crate::executions::swap;
use valkyrie::test_utils::expect_generic_err;
use terraswap::router::{ExecuteMsg, SwapOperation};
use terraswap::asset::AssetInfo;
use cw20::Cw20ExecuteMsg;
use valkyrie::test_constants::fund_manager::{fund_manager_env, FUND_MANAGER, MANAGING_TOKEN};
use valkyrie::test_constants::{default_sender, TERRASWAP_ROUTER};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
    route: Option<Vec<Denom>>,
) -> ContractResult<Response> {
    swap(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        denom,
        amount,
        route,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    denom: Denom,
    amount: Option<Uint128>,
    route: Option<Vec<Denom>>,
) -> (Env, MessageInfo, Response) {
    let env = fund_manager_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        denom,
        amount,
        route,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed_native() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_native_balance(FUND_MANAGER, vec![
        coin(10000u128, "uusd"),
    ]);

    let (_, _, response) = will_success(
        &mut deps,
        Denom::Native("uusd".to_string()),
        None,
        None,
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TERRASWAP_ROUTER.to_string(),
            funds: vec![coin(10000, "uusd")],
            msg: to_binary(&ExecuteMsg::ExecuteSwapOperations {
                operations: vec![
                    SwapOperation::TerraSwap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: "uusd".to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: Addr::unchecked(MANAGING_TOKEN),
                        },
                    },
                ],
                minimum_receive: None,
                to: None,
            }).unwrap(),
        })),
    ]);
}

#[test]
fn succeed_token() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        "Token1",
        &[(FUND_MANAGER, &Uint128::new(10000))],
    )]);

    super::instantiate::default(&mut deps);

    let (_, _, response) = will_success(
        &mut deps,
        Denom::Token("Token1".to_string()),
        None,
        None,
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "Token1".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: TERRASWAP_ROUTER.to_string(),
                amount: Uint128::new(10000),
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperations {
                    operations: vec![
                        SwapOperation::TerraSwap {
                            offer_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("Token1"),
                            },
                            ask_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked(MANAGING_TOKEN),
                            },
                        },
                    ],
                    minimum_receive: None,
                    to: None,
                }).unwrap(),
            }).unwrap(),
        })),
    ]);
}

#[test]
fn succeed_route() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_native_balance(FUND_MANAGER, vec![
        coin(10000u128, "ukrw"),
    ]);

    let (_, _, response) = will_success(
        &mut deps,
        Denom::Native("ukrw".to_string()),
        None,
        Some(vec![
            Denom::Native("ukrw".to_string()),
            Denom::Native("uusd".to_string()),
            Denom::Token(MANAGING_TOKEN.to_string()),
        ]),
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TERRASWAP_ROUTER.to_string(),
            funds: vec![coin(10000, "ukrw")],
            msg: to_binary(&ExecuteMsg::ExecuteSwapOperations {
                operations: vec![
                    SwapOperation::NativeSwap {
                        offer_denom: "ukrw".to_string(),
                        ask_denom: "uusd".to_string(),
                    },
                    SwapOperation::TerraSwap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: "uusd".to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: Addr::unchecked(MANAGING_TOKEN),
                        },
                    },
                ],
                minimum_receive: None,
                to: None,
            }).unwrap(),
        })),
    ]);
}

#[test]
fn failed_invalid_route() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_native_balance(FUND_MANAGER, vec![
        coin(10000u128, "ukrw"),
    ]);

    let result = exec(
        &mut deps,
        fund_manager_env(),
        default_sender(),
        Denom::Native("ukrw".to_string()),
        None,
        Some(vec![
            Denom::Native("ukrw".to_string()),
        ]),
    );
    expect_generic_err(
        &result,
        format!(
            "route must start with '{}' and end with '{}'",
            "ukrw".to_string(), MANAGING_TOKEN.to_string(),
        ).as_str(),
    );

    let result = exec(
        &mut deps,
        fund_manager_env(),
        default_sender(),
        Denom::Native("ukrw".to_string()),
        None,
        Some(vec![
            Denom::Native("uusd".to_string()),
            Denom::Token(MANAGING_TOKEN.to_string()),
        ]),
    );
    expect_generic_err(
        &result,
        format!(
            "route must start with '{}' and end with '{}'",
            "ukrw".to_string(), MANAGING_TOKEN.to_string(),
        ).as_str(),
    );

    let result = exec(
        &mut deps,
        fund_manager_env(),
        default_sender(),
        Denom::Native("ukrw".to_string()),
        None,
        Some(vec![
            Denom::Native("ukrw".to_string()),
            Denom::Native("uusd".to_string()),
        ]),
    );
    expect_generic_err(
        &result,
        format!(
            "route must start with '{}' and end with '{}'",
            "ukrw".to_string(), MANAGING_TOKEN.to_string(),
        ).as_str(),
    );
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    deps.querier.plus_native_balance(FUND_MANAGER, vec![
        coin(10000u128, "ukrw"),
    ]);
    deps.querier.with_token_balances(&[(
        "Token1",
        &[(FUND_MANAGER, &Uint128::new(10000))],
    )]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        fund_manager_env(),
        default_sender(),
        Denom::Native("ukrw".to_string()),
        Some(Uint128::new(10001)),
        None,
    );
    expect_generic_err(&result, "Insufficient balance");

    let result = exec(
        &mut deps,
        fund_manager_env(),
        default_sender(),
        Denom::Token("Token1".to_string()),
        Some(Uint128::new(10001)),
        None,
    );
    expect_generic_err(&result, "Insufficient balance");
}