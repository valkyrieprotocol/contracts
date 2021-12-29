use cosmwasm_std::{Addr, Uint128, Env, MessageInfo, Response};
use cosmwasm_std::testing::mock_info;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use valkyrie::test_constants::governance::{GOVERNANCE, governance_env, GOVERNANCE_TOKEN, TICKET_TOKEN};
use crate::staking::executions::{stake_governance_token_hook, unstake_governance_token_hook};
use crate::staking::states::{StakerState, StakingState};
use crate::tests::init_default;
use crate::vp::executions::{ticket_claim};
use valkyrie::message_matchers;
use crate::vp::queries::{get_ticket_staker_state};

pub const STAKER1:&str = "staker1";
pub const STAKER2:&str = "staker2";

pub fn stake(deps: &mut CustomDeps, env: Env, info: MessageInfo, sender: Addr, amount: Uint128) -> ContractResult<Response> {
    deps.querier.plus_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(GOVERNANCE, &amount)],
    )]);
    stake_governance_token_hook(deps.as_mut(), env, info, sender.to_string(), amount)
}

pub fn unstake(deps: &mut CustomDeps, env: Env, info: MessageInfo, staker: Addr, amount: Uint128) -> ContractResult<Response> {
    let response = unstake_governance_token_hook(
        deps.as_mut(),
        env,
        info,
        staker.to_string(),
        Some(amount),
    )?;

    for msg in message_matchers::cw20_transfer(&response.messages) {
        deps.querier.minus_token_balances(&[(
            &msg.contract_addr,
            &[(GOVERNANCE, &msg.amount)],
        )]);
        deps.querier.plus_token_balances(&[(
            &msg.contract_addr,
            &[(&msg.recipient, &msg.amount)],
        )]);
    }

    Ok(response)
}

#[test]
fn test_compute_reward() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());
    let mut env = governance_env();
    let info = mock_info(GOVERNANCE, &[]);

    env.block.height = 0;

    StakingState {
        total_share: Uint128::zero()
    }.save(deps.as_mut().storage).unwrap();

    StakerState {
        address: Addr::unchecked(STAKER1),
        share: Uint128::zero(),
        votes: vec![]
    }.save(deps.as_mut().storage).unwrap();

    StakerState {
        address: Addr::unchecked(STAKER2),
        share: Uint128::zero(),
        votes: vec![]
    }.save(deps.as_mut().storage).unwrap();

    stake(&mut deps, env.clone(), info.clone(), Addr::unchecked(STAKER1), Uint128::new(500u128)).unwrap();
    stake(&mut deps, env.clone(), info.clone(), Addr::unchecked(STAKER2), Uint128::new(50u128)).unwrap();
    //schedule : 0~100 / 100_000000
    //staker A : 500 staked
    //staker B : 50 staked

    env.block.height = 10;

    let res1 = get_ticket_staker_state(deps.as_ref(), env.clone(), Addr::unchecked(STAKER1).to_string()).unwrap();
    let res2 = get_ticket_staker_state(deps.as_ref(), env.clone(), Addr::unchecked(STAKER2).to_string()).unwrap();

    assert_eq!(res1.pending_reward, Uint128::new(9_090909u128));
    assert_eq!(res2.pending_reward, Uint128::new(909090u128));
    //staker A : 10*(500/550)
    //staker B : 10*(50/550)

    env.block.height = 50;

    let res1 = get_ticket_staker_state(deps.as_ref(), env.clone(), Addr::unchecked(STAKER1).to_string()).unwrap();
    let res2 = get_ticket_staker_state(deps.as_ref(), env.clone(), Addr::unchecked(STAKER2).to_string()).unwrap();
    assert_eq!(res1.pending_reward, Uint128::new(45_454545u128));
    assert_eq!(res2.pending_reward, Uint128::new(4_545454u128));
    //staker A : 50*(500/550)
    //staker B : 50*(50/550)

    let info = mock_info(STAKER1, &[]);
    let _res = ticket_claim(deps.as_mut(), env.clone(), info.clone()).unwrap();
    //staker A : all claimed

    let res1 = get_ticket_staker_state(deps.as_ref(), env.clone(), Addr::unchecked(STAKER1).to_string()).unwrap();
    let res2 = get_ticket_staker_state(deps.as_ref(), env.clone(), Addr::unchecked(STAKER2).to_string()).unwrap();
    assert_eq!(res1.pending_reward, Uint128::new(0_000000u128));
    assert_eq!(res2.pending_reward, Uint128::new(4_545454u128));
}
