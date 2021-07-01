use cosmwasm_std::{Addr, Env, MessageInfo, Response, Uint128};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::ContractResult;
use valkyrie::governance::enumerations::VoteOption;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, expect_generic_err};

use crate::poll::executions::cast_vote;
use crate::poll::states::{Poll, VoteInfo};
use crate::staking::states::StakerState;
use crate::tests::init_default;

pub const VOTER1: &str = "Voter1";
pub const VOTER2: &str = "Voter2";
pub const VOTER3: &str = "Voter3";

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    option: VoteOption,
    amount: Uint128,
) -> ContractResult<Response> {
    cast_vote(
        deps.as_mut(),
        env,
        info,
        poll_id,
        option,
        amount,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    voter: &str,
    poll_id: u64,
    option: VoteOption,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(voter, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        poll_id,
        option,
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, Uint128(100));

    let voter_addr = Addr::unchecked(VOTER1);
    let poll_id = 1u64;
    let vote_option = VoteOption::Yes;
    let vote_amount = Uint128(100);

    will_success(&mut deps, VOTER1, poll_id, vote_option.clone(), vote_amount);

    let vote_info = VoteInfo {
        voter: voter_addr.clone(),
        option: vote_option.clone(),
        amount: vote_amount,
    };

    let staker_state = StakerState::load(&deps.storage, &voter_addr).unwrap();
    assert_eq!(staker_state.votes, vec![(poll_id, vote_info.clone())]);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.yes_votes, vote_amount);

    let voter = poll.load_voter(&deps.storage, &voter_addr).unwrap();
    assert_eq!(voter, vote_info);
}

#[test]
fn failed_cast_vote_not_enough_staked() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let stake_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, stake_amount);

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(VOTER1, &[]),
        1,
        VoteOption::Yes,
        stake_amount + Uint128(1),
    );

    expect_generic_err(&result, "User does not have enough staked tokens.");
}

#[test]
fn failed_cast_vote_without_poll() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let stake_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, stake_amount);

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(VOTER1, &[]),
        0,
        VoteOption::Yes,
        stake_amount,
    );

    expect_generic_err(&result, "Poll does not exist");
}

#[test]
fn failed_cast_vote_twice() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, Uint128(100));

    let poll_id = 1u64;

    will_success(
        &mut deps,
        VOTER1,
        poll_id,
        VoteOption::Yes,
        Uint128(10),
    );

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(VOTER1, &[]),
        poll_id,
        VoteOption::Yes,
        Uint128(10),
    );

    expect_generic_err(&result, "User has already voted.");
}