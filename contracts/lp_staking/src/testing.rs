use crate::entrypoints::{execute, instantiate, query};
use crate::mock_querier::mock_dependencies_with_querier;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, QueryRequest, StdError, SubMsg, Uint128, WasmMsg, WasmQuery, Env, MessageInfo};
use cw20::{BalanceResponse, Cw20QueryMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use valkyrie::lp_staking::query_msgs::{
    ConfigResponse, QueryMsg, StakerInfoResponse, StateResponse,
};
use valkyrie::mock_querier::{custom_deps, CustomDeps};

fn init(
    deps: &mut CustomDeps,
    sender: String,
    whitelisted_contracts: Vec<String>,
    distribution_schedule: Vec<(u64, u64, Uint128)>,
) -> ContractResult<(Env, MessageInfo)> {
    let msg = InstantiateMsg {
        token: "reward0000".to_string(),
        lp_token: "lp_token".to_string(),
        pair: "pair".to_string(),
        whitelisted_contracts: whitelisted_contracts,
        distribution_schedule: distribution_schedule,
    };

    let mut env = mock_env();

    env.block.height = 0;

    let info = mock_info(Addr::unchecked(sender.as_str()).as_str(), &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg)?;


    Ok((env, info))
}


#[test]
fn proper_initialization() {
    let mut deps = custom_deps();
    let (env, _info) = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![(100, 200, Uint128::from(1000000u128))],
    ).unwrap();

    // it worked, let's query the state
    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            admin: "addr0000".to_string(),
            token: "reward0000".to_string(),
            pair: "pair".to_string(),
            lp_token: "lp_token".to_string(),
            whitelisted_contracts: vec!["contract1".to_string(), "contract2".to_string()],
            distribution_schedule: vec![(100, 200, Uint128::from(1000000u128))],
        }
    );

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::State { block_height: None },
    )
        .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            last_distributed: 0,
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        }
    );
}

#[test]
fn is_valid_distribution_schedule() {
    let mut deps = custom_deps();
    let res = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![(200, 200, Uint128::from(1000000u128))],
    ).unwrap_err();

    assert_eq!(res, ContractError::Std(StdError::generic_err(
        "invalid schedule",
    )));

    let res = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![(100, 200, Uint128::from(1000000u128)), (180, 200, Uint128::from(1000000u128))],
    ).unwrap_err();

    assert_eq!(res, ContractError::Std(StdError::generic_err(
        "invalid schedule",
    )));


    let (_env, _info) = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![(100, 200, Uint128::from(1000000u128)), (200, 300, Uint128::from(1000000u128))],
    ).unwrap();
}

#[test]
fn update_config() {
    let mut deps = custom_deps();

    let (env, _info) = init(
        &mut deps,
        "admin".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![(100, 200, Uint128::from(1000000u128))],
    ).unwrap();

    let msg = ExecuteMsg::UpdateConfig {
        whitelisted_contracts: None,
        distribution_schedule: Some(vec![(200, 300, Uint128::from(1000000u128))]),
    };

    let info = mock_info(Addr::unchecked("not_admin").as_str(), &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});

    let msg = ExecuteMsg::UpdateConfig {
        whitelisted_contracts: None,
        distribution_schedule: Some(vec![(300, 400, Uint128::from(1000000u128))]),
    };

    let info = mock_info(Addr::unchecked("admin").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            admin: "admin".to_string(),
            token: "reward0000".to_string(),
            pair: "pair".to_string(),
            lp_token: "lp_token".to_string(),
            whitelisted_contracts: vec!["contract1".to_string(), "contract2".to_string()],
            distribution_schedule: vec![(300, 400, Uint128::from(1000000u128))],
        }
    );


    let msg = ExecuteMsg::UpdateConfig {
        whitelisted_contracts: Some(vec!["contract3".to_string(), "contract4".to_string()]),
        distribution_schedule: None,
    };

    let info = mock_info(Addr::unchecked("admin").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            admin: "admin".to_string(),
            token: "reward0000".to_string(),
            pair: "pair".to_string(),
            lp_token: "lp_token".to_string(),
            whitelisted_contracts: vec!["contract3".to_string(), "contract4".to_string()],
            distribution_schedule: vec![(300, 400, Uint128::from(1000000u128))],
        }
    );
}

fn query_cw20_balance(deps: &CustomDeps, contract: String, address: String) -> Uint128 {
    let balance = &deps
        .querier
        .handle_query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract,
            msg: to_binary(&Cw20QueryMsg::Balance { address: address }).unwrap(),
        }))
        .unwrap();

    let bbb: BalanceResponse = from_binary(&balance.clone().unwrap()).unwrap();
    bbb.balance
}

#[test]
fn migrate_reward() {
    let mut deps = custom_deps();

    let msg = InstantiateMsg {
        token: "reward0000".to_string(),
        lp_token: "lp_token".to_string(),
        pair: "pair".to_string(),
        whitelisted_contracts: vec!["contract1".to_string(), "contract2".to_string()],
        distribution_schedule: vec![(100, 200, Uint128::from(1000000u128))],
    };

    let mut env = mock_env();
    env.block.height = 0;
    let info = mock_info(Addr::unchecked("admin").as_str(), &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    deps.querier.plus_token_balances(&[(
        "reward0000",
        &[(env.contract.address.as_str(), &Uint128::new(123456u128))],
    )]);

    let balance = query_cw20_balance(
        &deps,
        "reward0000".to_string(),
        env.contract.address.to_string(),
    );
    assert_eq!(balance, Uint128::new(123456u128));

    let msg = ExecuteMsg::MigrateReward {
        recipient: "admin2".to_string(),
        amount: Uint128::new(1234u128),
    };

    let info = mock_info(Addr::unchecked("not_admin").as_str(), &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

    assert_eq!(res, ContractError::Unauthorized {});

    // let msg = ExecuteMsg::MigrateReward {
    //     recipient: "admin2".to_string(),
    // };

    // let info = mock_info(Addr::unchecked("admin").as_str(), &[]);
    // let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // let balance = query_cw20_balance(
    //     &deps,
    //     "reward0000".to_string(),
    //     env.contract.address.to_string(),
    // );
    // assert_eq!(balance, Uint128::zero());
    // let balance = query_cw20_balance(&deps, "reward0000".to_string(), "admin2".to_string());
    // assert_eq!(balance, Uint128::new(123456u128));
}

#[test]
fn test_bond_tokens() {
    let mut deps = custom_deps();

    let (mut env, _info) = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![
            (0, 100, Uint128::from(1000000u128)),
            (100, 200, Uint128::from(10000000u128)),
        ],
    ).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });

    let info = mock_info(Addr::unchecked("lp_token").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string(),
                },
            )
                .unwrap(),
        )
            .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::zero(),
            pending_reward: Uint128::zero(),
            bond_amount: Uint128::new(100u128),
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::State { block_height: None },
            )
                .unwrap()
        )
            .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::new(100u128),
            global_reward_index: Decimal::zero(),
            last_distributed: 0,
        }
    );

    // bond 100 more tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    env.block.height += 10;
    //now block : 10;

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string(),
                },
            )
                .unwrap(),
        )
            .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(1000u128, 1u128),
            pending_reward: Uint128::from(100000u128),
            bond_amount: Uint128::new(200u128),
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::State { block_height: None },
            )
                .unwrap()
        )
            .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::new(200u128),
            global_reward_index: Decimal::from_ratio(1000u128, 1u128),
            last_distributed: 10,
        }
    );

    let msg = ExecuteMsg::UpdateConfig {
        whitelisted_contracts: None,
        distribution_schedule: Some(vec![
            (10, 200, Uint128::from(9500u128)),
            (200, 400, Uint128::from(100000u128)),
        ]),
    };

    let info = mock_info(Addr::unchecked("addr0000").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let _res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();


    env.block.height += 10;
    //now block 20

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string(),
                },
            )
                .unwrap(),
        )
            .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(10025u128, 10u128),
            pending_reward: Uint128::from(100500u128),
            bond_amount: Uint128::new(200u128),
        }
    );


    // failed with unautorized
    // let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
    //     sender: Addr::unchecked("addr0000").to_string(),
    //     amount: Uint128::new(100u128),
    //     msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    // });

    // let env = mock_env();
    // let info = mock_info(Addr::unchecked("staking0001").as_str(), &[]);
    // let res = execute(deps.as_mut(), env, info, msg);
    // match res {
    //     Err(StdError::generic_err) => {}
    //     _ => panic!("Must return unauthorized error"),
    // }
}

#[test]
fn test_unbond() {
    let mut deps = custom_deps();

    let (env, _info) = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![
            (0, 100, Uint128::from(1000000u128)),
            (100, 200, Uint128::from(10000000u128)),
        ],
    ).unwrap();

    // bond 100 tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: Addr::unchecked("addr0000").to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info(Addr::unchecked("lp_token").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // unbond 150 tokens; failed
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(150u128),
    };

    let info = mock_info(Addr::unchecked("addr0000").as_str(), &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();

    assert_eq!(
        res,
        ContractError::Std(StdError::generic_err("Cannot unbond more than bond amount"))
    );

    // normal unbond
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(100u128),
    };

    let info = mock_info(Addr::unchecked("addr0000").as_str(), &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: Addr::unchecked("lp_token").to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: Addr::unchecked("addr0000").to_string(),
                amount: Uint128::new(100u128),
            })
                .unwrap(),
            funds: vec![],
        }))]
    );
}

#[test]
fn test_compute_reward() {
    let mut deps = custom_deps();

    let (mut env, _info) = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![
            (0, 100, Uint128::from(1000000u128)),
            (100, 200, Uint128::from(10000000u128)),
        ],
    ).unwrap();

    // bond 100 tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: Addr::unchecked("addr0000").to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let mut info = mock_info(Addr::unchecked("lp_token").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // 100 blocks passed
    // 1,000,000 rewards distributed
    env.block.height += 100;

    // bond 100 more tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: Addr::unchecked("addr0000").to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: Addr::unchecked("addr0000").to_string(),
                },
            )
                .unwrap()
        )
            .unwrap(),
        StakerInfoResponse {
            staker: Addr::unchecked("addr0000").to_string(),
            reward_index: Decimal::from_ratio(10000u128, 1u128),
            pending_reward: Uint128::new(1000000u128),
            bond_amount: Uint128::new(200u128),
        }
    );

    // 100 blocks passed
    // 1,000,000 rewards distributed
    env.block.height += 10;
    info.sender = Addr::unchecked("addr0000");

    // unbond
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(100u128),
    };
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: Addr::unchecked("addr0000").to_string(),
                },
            )
                .unwrap()
        )
            .unwrap(),
        StakerInfoResponse {
            staker: Addr::unchecked("addr0000").to_string(),
            reward_index: Decimal::from_ratio(15000u64, 1u64),
            pending_reward: Uint128::new(2000000u128),
            bond_amount: Uint128::new(100u128),
        }
    );

    // query future block

    env.block.height = 120;

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: Addr::unchecked("addr0000").to_string(),
                },
            )
                .unwrap()
        )
            .unwrap(),
        StakerInfoResponse {
            staker: Addr::unchecked("addr0000").to_string(),
            reward_index: Decimal::from_ratio(25000u64, 1u64),
            pending_reward: Uint128::new(3000000u128),
            bond_amount: Uint128::new(100u128),
        }
    );
}


#[test]
fn test_withdraw() {
    let mut deps = custom_deps();

    let (mut env, _info) = init(
        &mut deps,
        "addr0000".to_string(),
        vec!["contract1".to_string(), "contract2".to_string()],
        vec![
            (0, 100, Uint128::from(1000000u128)),
            (100, 200, Uint128::from(10000000u128)),
        ],
    ).unwrap();

    // bond 100 tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: Addr::unchecked("addr0000").to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let mut info = mock_info(Addr::unchecked("lp_token").as_str(), &[]);
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // 100 blocks passed
    // 1,000,000 rewards distributed
    env.block.height += 100;
    info.sender = Addr::unchecked("addr0000");

    let msg = ExecuteMsg::Withdraw {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: Addr::unchecked("reward0000").to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: Addr::unchecked("addr0000").to_string(),
                amount: Uint128::new(1000000u128),
            })
                .unwrap(),
            funds: vec![],
        }))]
    );
}

#[test]
fn test_auto_stake() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let init = InstantiateMsg {
        token: "asset".to_string(),
        lp_token: "lp_token".to_string(),
        pair: "pair".to_string(),
        whitelisted_contracts: vec!["contract1".to_string(), "contract2".to_string()],
        distribution_schedule: vec![
            (12345, 12345 + 100, Uint128::from(1000000u128)),
            (12345 + 100, 12345 + 200, Uint128::from(10000000u128)),
        ],
    };

    // check, ust funds.
    let msg = ExecuteMsg::AutoStake {
        token_amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };
    let env = mock_env();
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
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), init).unwrap();
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::Std(StdError::generic_err("UST only")));

    // check, ust funds.
    let msg = ExecuteMsg::AutoStake {
        token_amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };
    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "ukrw".to_string(),
            amount: Uint128::new(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::Std(StdError::generic_err("UST only")));

    // check, ust funds.
    let msg = ExecuteMsg::AutoStake {
        token_amount: Uint128::from(100u64),
        slippage_tolerance: None,
    };
    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::zero(),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(StdError::generic_err("Send UST more than zero"))
    );

    let msg = ExecuteMsg::AutoStake {
        token_amount: Uint128::from(1u64),
        slippage_tolerance: None,
    };

    // check, ust+token -> LP -> staking.
    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: Addr::unchecked("asset").to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: Addr::unchecked("addr0000").to_string(),
                    recipient: Addr::unchecked(MOCK_CONTRACT_ADDR).to_string(),
                    amount: Uint128::new(1u128),
                })
                    .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: Addr::unchecked("asset").to_string(),
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: Addr::unchecked("pair").to_string(),
                    amount: Uint128::new(1),
                    expires: None,
                })
                    .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: Addr::unchecked("pair").to_string(),
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
                                contract_addr: Addr::unchecked("asset").to_string()
                            },
                            amount: Uint128::new(1u128),
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
                contract_addr: Addr::unchecked(MOCK_CONTRACT_ADDR).to_string(),
                msg: to_binary(&ExecuteMsg::AutoStakeHook {
                    staker_addr: Addr::unchecked("addr0000").to_string(),
                    already_staked_amount: Uint128::new(0),
                })
                    .unwrap(),
                funds: vec![],
            })),
        ]
    );
}
