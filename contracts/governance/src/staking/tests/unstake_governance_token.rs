use cosmwasm_std::{Addr, Env, MessageInfo, Response, Uint128};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::governance::{GOVERNANCE, GOVERNANCE_TOKEN, governance_env};
use valkyrie::test_utils::expect_generic_err;
use valkyrie::utils::parse_uint128;

use crate::staking::executions::unstake_governance_token;
use crate::staking::states::{StakerState, StakingState};
use crate::staking::tests::stake_governance_token::{STAKER1, STAKER1_STAKE_AMOUNT, STAKER2, STAKER2_STAKE_AMOUNT};
use crate::tests::init_default;
use valkyrie::test_constants::default_sender;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, amount: Option<Uint128>) -> ContractResult<Response> {
    unstake_governance_token(deps.as_mut(), env, info, amount)
}

pub fn will_success(deps: &mut CustomDeps, staker: &str, amount: Option<Uint128>) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = mock_info(staker, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::stake_governance_token::will_success(&mut deps, STAKER1, STAKER1_STAKE_AMOUNT);
    super::stake_governance_token::will_success(&mut deps, STAKER2, STAKER2_STAKE_AMOUNT);

    let increased_balance = (STAKER1_STAKE_AMOUNT + STAKER2_STAKE_AMOUNT)
        .checked_mul(Uint128::new(2))
        .unwrap();

    deps.querier.with_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(GOVERNANCE, &increased_balance)]
    )]);

    let (_, _, response) = will_success(&mut deps, STAKER1, None);

    let unstake_amount = response.attributes.iter()
        .find_map(|v| if v.key == "unstake_amount" {
            Some(parse_uint128(&v.value).unwrap())
        } else {
            None
        })
        .unwrap();

    let unstake_share = response.attributes.iter()
        .find_map(|v| if v.key == "unstake_share" {
            Some(parse_uint128(&v.value).unwrap())
        } else {
            None
        })
        .unwrap();

    let staking_state = StakingState::load(&deps.storage).unwrap();
    let staker_state = StakerState::load(&deps.storage, &Addr::unchecked(STAKER1)).unwrap();

    assert_eq!(unstake_amount, STAKER1_STAKE_AMOUNT.checked_mul(Uint128::new(2)).unwrap());
    assert_eq!(unstake_share, STAKER1_STAKE_AMOUNT);
    assert_eq!(staking_state.total_share, STAKER2_STAKE_AMOUNT);
    assert_eq!(staker_state.share, Uint128::zero());
}

#[test]
fn remove_completed_vote() {
    //TODO: Implement after poll test codes.
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::stake_governance_token::will_success(&mut deps, STAKER1, STAKER1_STAKE_AMOUNT);

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(STAKER1, &[]),
        Some(STAKER1_STAKE_AMOUNT + Uint128::new(1)),
    );

    expect_generic_err(&result, "User is trying to unstake too many tokens.")
}

#[test]
fn failed_no_staked() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        None,
    );

    expect_generic_err(&result, "Nothing staked");
}