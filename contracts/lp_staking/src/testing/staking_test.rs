use crate::entrypoints::{execute, instantiate, query};
use crate::states::State;
use crate::testing::mock_querier::mock_dependencies_with_querier;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, StdError, SubMsg, Uint128,
    WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use valkyrie::lp_staking::query_msgs::{QueryMsg, StakerInfoResponse};

#[test]
fn test_bond_tokens() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        token: "reward".to_string(),
        pair: "pair".to_string(),
        lp_token: "lp_token".to_string(),
    };

    let info = mock_info("addr", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });

    let info = mock_info("lp_token", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::StakerInfo {
            staker_addr: "addr".to_string(),
        },
    )
    .unwrap();
    let res: StakerInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        StakerInfoResponse {
            staker_addr: "addr".to_string(),
            pending_reward: Uint128::zero(),
            bond_amount: Uint128::new(100u128),
        }
    );

    let data = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();

    let state: State = from_binary(&data).unwrap();
    assert_eq!(
        state,
        State {
            total_bond_amount: Uint128::new(100u128),
            global_reward_index: Decimal::zero(),
            pending_reward: Uint128::zero(),
        }
    );

    // bond 100 more tokens from other account
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr2".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info("lp_token", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let data = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&data).unwrap();
    assert_eq!(
        state,
        State {
            total_bond_amount: Uint128::new(200u128),
            global_reward_index: Decimal::zero(),
            pending_reward: Uint128::zero(),
        }
    );

    // failed with unauthorized
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });

    let info = mock_info("staking2", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn test_unbond() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        token: "reward".to_string(),
        pair: "pair".to_string(),
        lp_token: "lp_token".to_string(),
    };

    let info = mock_info("addr", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // bond 100 tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info("lp_token", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unbond 150 tokens; failed
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(150u128),
    };

    let info = mock_info("addr", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "Cannot unbond more than bond amount");
        }
        _ => panic!("Must return generic error"),
    };

    // normal unbond
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(100u128),
    };

    let info = mock_info("addr", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lp_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr".to_string(),
                amount: Uint128::new(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    let data = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&data).unwrap();
    assert_eq!(
        state,
        State {
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            pending_reward: Uint128::zero(),
        }
    );

    let data = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::StakerInfo {
            staker_addr: "addr".to_string(),
        },
    )
    .unwrap();
    let res: StakerInfoResponse = from_binary(&data).unwrap();
    assert_eq!(
        res,
        StakerInfoResponse {
            staker_addr: "addr".to_string(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
        }
    );
}

#[test]
fn test_auto_stake() {
    let mut deps = mock_dependencies_with_querier(&[]);
    deps.querier.with_pair_info(Addr::unchecked("pair"));
    deps.querier.with_pool_assets([
        Asset {
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            amount: Uint128::from(100u128),
        },
        Asset {
            info: AssetInfo::Token {
                contract_addr: "asset".to_string(),
            },
            amount: Uint128::from(1u128),
        },
    ]);

    let msg = InstantiateMsg {
        token: "token".to_string(),
        pair: "pair".to_string(),
        lp_token: "lp_token".to_string(),
    };
    let info = mock_info("addr", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // token > 0
    let msg = ExecuteMsg::AutoStake {
        amount: Uint128::zero(),
        slippage_tolerance: None,
    };
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(100u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("token amount > 0"));

    // only uusd
    let msg = ExecuteMsg::AutoStake {
        amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };
    let info = mock_info(
        "addr0000",
        &[
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100u128),
            },
            Coin {
                denom: "ukrw".to_string(),
                amount: Uint128::new(100u128),
            },
        ],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("must uusd only"));

    let msg = ExecuteMsg::AutoStake {
        amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "ukrw".to_string(),
            amount: Uint128::new(100u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("must uusd"));

    // no native asset
    let msg = ExecuteMsg::AutoStake {
        amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(res, StdError::generic_err("must uusd only"));

    let msg = ExecuteMsg::AutoStake {
        amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(100u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: "addr0000".to_string(),
                    recipient: MOCK_CONTRACT_ADDR.to_string(),
                    amount: Uint128::new(100u128),
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: "pair".to_string(),
                    amount: Uint128::new(100),
                    expires: None,
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "pair".to_string(),
                msg: to_binary(&PairExecuteMsg::ProvideLiquidity {
                    assets: [
                        Asset {
                            info: AssetInfo::NativeToken {
                                denom: "uusd".to_string()
                            },
                            amount: Uint128::new(99u128),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: "token".to_string()
                            },
                            amount: Uint128::new(100u128),
                        },
                    ],
                    slippage_tolerance: None,
                    receiver: None,
                })
                .unwrap(),
                funds: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(99u128), // 1% tax
                }],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                msg: to_binary(&ExecuteMsg::AutoStakeHook {
                    staker_addr: "addr0000".to_string(),
                    prev_staking_token_amount: Uint128::new(0),
                })
                .unwrap(),
                funds: vec![],
            }))
        ]
    );

    deps.querier.with_token_balance(Uint128::new(100u128)); // recive 100 lptoken

    // valid msg
    let msg = ExecuteMsg::AutoStakeHook {
        staker_addr: "addr0000".to_string(),
        prev_staking_token_amount: Uint128::new(0),
    };

    // unauthorized attempt
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    assert_eq!(res, StdError::generic_err("unauthorized"));

    // successfull attempt
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "bond"),
            attr("staker_addr", "addr0000"),
            attr("amount", "100"),
        ]
    );

    let data = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&data).unwrap();
    assert_eq!(
        state,
        State {
            total_bond_amount: Uint128::new(100u128),
            global_reward_index: Decimal::zero(),
            pending_reward: Uint128::zero(),
        }
    );
}
