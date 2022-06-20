use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, coin, CosmosMsg, WasmMsg, to_binary, Addr, SubMsg};
use valkyrie::common::{ContractResult, Denom};
use crate::executions::swap_fee;
use valkyrie::test_utils::expect_generic_err;
use cw20::Cw20ExecuteMsg;
use valkyrie::proxy::asset::AssetInfo;
use valkyrie::proxy::execute_msgs::{ExecuteMsg, SwapOperation};
use valkyrie::test_constants::{default_sender, VALKYRIE_PROXY, VALKYRIE_TOKEN};
use valkyrie::test_constants::campaign_manager::{CAMPAIGN_MANAGER, campaign_manager_env};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
    route: Option<Vec<Denom>>,
) -> ContractResult<Response> {
    swap_fee(
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
    let env = campaign_manager_env();
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

    deps.querier.plus_native_balance(CAMPAIGN_MANAGER, vec![
        coin(10000u128, "uluna"),
    ]);

    let (_, _, response) = will_success(
        &mut deps,
        Denom::Native("uluna".to_string()),
        None,
        None,
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_PROXY.to_string(),
            funds: vec![coin(10000, "uluna")],
            msg: to_binary(&ExecuteMsg::ExecuteSwapOperations {
                operations: vec![
                    SwapOperation::Swap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: "uluna".to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: Addr::unchecked(VALKYRIE_TOKEN).to_string(),
                        },
                    },
                ],
                minimum_receive: None,
                to: None,
                max_spread: None,
            }).unwrap(),
        })),
    ]);
}

#[test]
fn succeed_token() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n",
        &[(CAMPAIGN_MANAGER, &Uint128::new(10000))],
    )]);

    super::instantiate::default(&mut deps);

    let (_, _, response) = will_success(
        &mut deps,
        Denom::Token("terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n".to_string()),
        None,
        None,
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: VALKYRIE_PROXY.to_string(),
                amount: Uint128::new(10000),
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperations {
                    operations: vec![
                        SwapOperation::Swap {
                            offer_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n").to_string(),
                            },
                            ask_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked(VALKYRIE_TOKEN).to_string(),
                            },
                        },
                    ],
                    minimum_receive: None,
                    to: None,
                    max_spread: None,
                }).unwrap(),
            }).unwrap(),
        })),
    ]);
}

#[test]
fn succeed_route() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_native_balance(CAMPAIGN_MANAGER, vec![
        coin(10000u128, "uluna"),
    ]);

    let (_, _, response) = will_success(
        &mut deps,
        Denom::Native("uluna".to_string()),
        None,
        Some(vec![
            Denom::Native("uluna".to_string()),
            Denom::Token(VALKYRIE_TOKEN.to_string()),
        ]),
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_PROXY.to_string(),
            funds: vec![coin(10000, "uluna")],
            msg: to_binary(&ExecuteMsg::ExecuteSwapOperations {
                operations: vec![
                    SwapOperation::Swap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: "uluna".to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: Addr::unchecked(VALKYRIE_TOKEN).to_string(),
                        },
                    },
                ],
                minimum_receive: None,
                to: None,
                max_spread: None,
            }).unwrap(),
        })),
    ]);
}

#[test]
fn failed_invalid_route() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_native_balance(CAMPAIGN_MANAGER, vec![
        coin(10000u128, "uluna"),
    ]);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
        Denom::Native("uluna".to_string()),
        None,
        Some(vec![
            Denom::Native("uluna".to_string()),
        ]),
    );
    expect_generic_err(
        &result,
        format!(
            "route must start with '{}' and end with '{}'",
            "uluna".to_string(), VALKYRIE_TOKEN.to_string(),
        ).as_str(),
    );
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    deps.querier.plus_native_balance(CAMPAIGN_MANAGER, vec![
        coin(10000u128, "ukrw"),
    ]);
    deps.querier.with_token_balances(&[(
        "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n",
        &[(CAMPAIGN_MANAGER, &Uint128::new(10000))],
    )]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
        Denom::Native("ukrw".to_string()),
        Some(Uint128::new(10001)),
        None,
    );
    expect_generic_err(&result, "Insufficient balance");

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
        Denom::Token("terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n".to_string()),
        Some(Uint128::new(10001)),
        None,
    );
    expect_generic_err(&result, "Insufficient balance");
}