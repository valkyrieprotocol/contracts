use cosmwasm_std::{Addr, CosmosMsg, Env, from_binary, MessageInfo, Response, Uint128, WasmMsg};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::governance::{governance_env_height, WITHDRAW_DELAY};
use valkyrie::test_utils::plus_height;

use crate::staking::executions::withdraw_governance_token;
use crate::staking::states::StakerState;
use crate::staking::tests::stake_governance_token::STAKER1;
use crate::tests::init_default;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    withdraw_governance_token(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, block_height: u64, staker: &str) -> (Env, MessageInfo, Response) {
    let env = governance_env_height(block_height);
    let info = mock_info(staker, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let (env, _, _) = super::stake_governance_token::will_success(
        &mut deps,
        STAKER1,
        Uint128::new(100),
    );

    let mut env = env.clone();
    let info = mock_info(STAKER1, &[]);

    plus_height(&mut env, 2);
    super::unstake_governance_token::exec(&mut deps, env.clone(), info.clone(), Some(Uint128::new(10))).unwrap();

    plus_height(&mut env, 2);
    super::unstake_governance_token::exec(&mut deps, env.clone(), info.clone(), Some(Uint128::new(10))).unwrap();

    plus_height(&mut env, 2);
    super::unstake_governance_token::exec(&mut deps, env.clone(), info.clone(), Some(Uint128::new(10))).unwrap();

    plus_height(&mut env, WITHDRAW_DELAY - 2);

    let mut withdrawable_amount = Uint128::zero();
    let staker_state = StakerState::load(&deps.storage, &Addr::unchecked(STAKER1)).unwrap();
    for (unlock_block, unstake_amount) in staker_state.unstaking_amounts {
        if env.block.height > unlock_block {
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

    match &response.messages[0].msg {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr: _, funds: _, msg }) => {
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