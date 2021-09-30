use crate::entrypoints::{execute, instantiate, query};
use crate::states::{StakerInfo, State};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Addr, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use valkyrie::lp_staking::query_msgs::{QueryMsg, StakerInfoResponse};

#[test]
fn test_deposit_reward() {
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

    // factory deposit 100 reward tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "factory".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::DepositReward {}).unwrap(),
    });
    let info = mock_info("reward", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // Check pool state
    let res: State =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()).unwrap();
    let res_cmp = res.clone();
    assert_eq!(
        res_cmp,
        State {
            total_bond_amount: Uint128::new(100u128),
            global_reward_index: Decimal::from_ratio(100u128, 100u128),
            ..res
        }
    );
}

#[test]
fn test_deposit_reward_when_no_bonding() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        token: "reward".to_string(),
        pair: "pair".to_string(),
        lp_token: "lp_token".to_string(),
    };

    let info = mock_info("addr", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // factory deposit 100 reward tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "factory".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::DepositReward {}).unwrap(),
    });
    let info = mock_info("reward", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // Check pool state
    let res: State =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()).unwrap();
    let res_cmp = res.clone();
    assert_eq!(
        res_cmp,
        State {
            global_reward_index: Decimal::zero(),
            pending_reward: Uint128::new(100u128),
            ..res
        }
    );
}

#[test]
fn test_before_share_changes() {
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

    // factory deposit 100 reward tokens
    // premium is 0, so rewards distributed 80:20
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "factory".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::DepositReward {}).unwrap(),
    });

    let info = mock_info("reward", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let addr = Addr::unchecked("addr".to_string());
    let staker_info: StakerInfo =
        StakerInfo::load_or_default(&deps.storage, &Addr::unchecked("addr".to_string())).unwrap();

    assert_eq!(
        StakerInfo {
            owner: addr,
            pending_reward: Uint128::zero(),
            bond_amount: Uint128::new(100u128),
            reward_index: Decimal::zero(),
        },
        staker_info
    );

    // bond 100 more tokens
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info("lp_token", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let staker_info: StakerInfo =
        StakerInfo::load_or_default(&deps.storage, &Addr::unchecked("addr".to_string())).unwrap();

    assert_eq!(
        StakerInfo {
            owner: Addr::unchecked("addr".to_string()),
            pending_reward: Uint128::new(100u128),
            bond_amount: Uint128::new(200u128),
            reward_index: Decimal::from_ratio(100u128, 100u128),
        },
        staker_info
    );

    // factory deposit 100 reward tokens; = 0.8 + 0.4 = 1.2 is reward_index
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "factory".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::DepositReward {}).unwrap(),
    });
    let info = mock_info("reward", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unbond
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(100u128),
    };
    let info = mock_info("addr", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let staker_info: StakerInfo =
        StakerInfo::load_or_default(&deps.storage, &Addr::unchecked("addr".to_string())).unwrap();

    assert_eq!(
        StakerInfo {
            owner: Addr::unchecked("addr".to_string()),
            pending_reward: Uint128::new(200u128),
            bond_amount: Uint128::new(100u128),
            reward_index: Decimal::from_ratio(150u128, 100u128),
        },
        staker_info
    );
}

#[test]
fn test_withdraw() {
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

    // factory deposit 100 reward tokens
    // premium_rate is zero; distribute weight => 80:20
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "factory".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::DepositReward {}).unwrap(),
    });
    let info = mock_info("reward", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Withdraw {};
    let info = mock_info("addr", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "reward".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr".to_string(),
                amount: Uint128::new(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );
}

#[test]
fn withdraw_multiple_rewards() {
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

    // factory deposit asset
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "factory".to_string(),
        amount: Uint128::new(100u128),
        msg: to_binary(&Cw20HookMsg::DepositReward {}).unwrap(),
    });
    let info = mock_info("reward", &[]);
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
            bond_amount: Uint128::new(100u128),
            pending_reward: Uint128::new(100u128),
        }
    );

    // withdraw all
    let msg = ExecuteMsg::Withdraw {};
    let info = mock_info("addr", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "reward".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr".to_string(),
                amount: Uint128::new(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))]
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
            bond_amount: Uint128::new(100u128),
            pending_reward: Uint128::zero(),
        }
    );
}
