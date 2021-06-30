use cosmwasm_std::{Addr, CosmosMsg, Env, from_binary, MessageInfo, Response, Uint128, WasmMsg};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use crate::staking::executions::withdraw_voting_token;
use crate::staking::states::StakerState;
use crate::staking::tests::stake::STAKER1;
use crate::tests::{default_env, env_plus_height, init_default, WITHDRAW_DELAY};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    withdraw_voting_token(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, block_height: u64, staker: &str) -> (Env, MessageInfo, Response) {
    let mut env = default_env();
    let info = mock_info(staker, &[]);

    let height_diff = (block_height - env.block.height) as i64;
    env_plus_height(&mut env, height_diff);
    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let (env, _, _) = super::stake::will_success(
        &mut deps,
        STAKER1,
        Uint128(100),
    );

    let mut env = env.clone();
    let info = mock_info(STAKER1, &[]);

    env_plus_height(&mut env, 2);
    super::unstake::exec(&mut deps, env.clone(), info.clone(), Some(Uint128(10))).unwrap();

    env_plus_height(&mut env, 2);
    super::unstake::exec(&mut deps, env.clone(), info.clone(), Some(Uint128(10))).unwrap();

    env_plus_height(&mut env, 2);
    super::unstake::exec(&mut deps, env.clone(), info.clone(), Some(Uint128(10))).unwrap();

    env_plus_height(&mut env, (WITHDRAW_DELAY - 2) as i64);

    let mut withdrawable_amount = Uint128::zero();
    let staker_state = StakerState::load(&deps.storage, &Addr::unchecked(STAKER1)).unwrap();
    for (unstake_block, unstake_amount) in staker_state.unstaking_amounts {
        if env.block.height > unstake_block + WITHDRAW_DELAY {
            withdrawable_amount += unstake_amount;
        }
    }

    let (_, _, response) = will_success(&mut deps, env.block.height, info.sender.as_str());

    let staker_state = StakerState::load(&deps.storage, &Addr::unchecked(STAKER1)).unwrap();

    for (unstake_block, _) in staker_state.unstaking_amounts {
        if env.block.height > unstake_block + WITHDRAW_DELAY {
            panic!("All unlocked amount should be withdraw")
        }
    }

    match &response.messages[0] {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr: _, send: _, msg }) => {
            match from_binary(msg).unwrap() {
                Cw20ExecuteMsg::Transfer { recipient, amount } => {
                    assert_eq!(recipient, info.sender.to_string());
                    assert_eq!(amount, withdrawable_amount);
                }
                _ => panic!("Unexpected wasm execute msg"),
            }
        }
        _ => panic!("Unexpected cosmos msg"),
    };
}