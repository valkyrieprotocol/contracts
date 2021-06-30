use valkyrie::mock_querier::{custom_deps, CustomDeps};
use crate::tests::{init_default, default_env, POLL_SNAPSHOT_PERIOD, default_info, expect_generic_err, env_set_height};
use cosmwasm_std::{Uint128, Env, MessageInfo, Response, attr};
use crate::poll::tests::cast_vote::{VOTER1, VOTER2, VOTER3};
use valkyrie::governance::enumerations::VoteOption;
use crate::poll::states::Poll;
use cosmwasm_std::testing::mock_info;
use valkyrie::common::ContractResult;
use crate::poll::executions::snapshot_poll;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    snapshot_poll(deps.as_mut(), env, info, poll_id)
}

pub fn will_success(deps: &mut CustomDeps, poll_id: u64) -> (Env, MessageInfo, Response) {
    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let mut env = default_env();
    env_set_height(&mut env, poll.end_height - 1);

    let info = default_info();

    let response = exec(deps, env.clone(), info.clone(), poll_id).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let voter1_staked_amount = Uint128(100);
    let voter2_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, voter1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, voter2_staked_amount);

    let poll_id = 1;

    let result = exec(&mut deps, default_env(), default_info(), poll_id);
    expect_generic_err(&result, "Cannot snapshot at this height");

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert_eq!(response.attributes, vec![
        attr("action", "snapshot_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("staked_amount", (voter1_staked_amount + voter2_staked_amount).to_string()),
    ]);
}

#[test]
fn succeed_within_cast_vote() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let voter1_staked_amount = Uint128(100);
    let voter2_staked_amount = Uint128(100);
    let voter3_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, voter1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, voter2_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, voter1_staked_amount);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let mut env = default_env();
    env_set_height(&mut env, poll.end_height - POLL_SNAPSHOT_PERIOD + 1);

    super::cast_vote::exec(
        &mut deps,
        env.clone(),
        mock_info(VOTER2, &[]),
        poll_id,
        VoteOption::Yes,
        Uint128(10),
    ).unwrap();

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.snapped_staked_amount, Some(voter1_staked_amount + voter2_staked_amount));

    let result = exec(&mut deps, env.clone(), default_info(), poll_id);
    expect_generic_err(&result, "Snapshot has already occurred");

    crate::staking::tests::stake::will_success(&mut deps, VOTER3, voter3_staked_amount);

    super::cast_vote::exec(
        &mut deps,
        env,
        mock_info(VOTER3, &[]),
        poll_id,
        VoteOption::No,
        Uint128(50),
    ).unwrap();

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.snapped_staked_amount, Some(voter1_staked_amount + voter2_staked_amount));
}

#[test]
fn failed_twice() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let voter1_staked_amount = Uint128(100);
    let voter2_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, voter1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, voter2_staked_amount);

    let poll_id = 1;
    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let mut env = default_env();
    env_set_height(&mut env, poll.end_height - 1);

    let response = exec(&mut deps, env.clone(), default_info(), poll_id).unwrap();
    assert_eq!(response.attributes, vec![
        attr("action", "snapshot_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("staked_amount", (voter1_staked_amount + voter2_staked_amount).to_string()),
    ]);

    let result = exec(&mut deps, env, default_info(), poll_id);
    expect_generic_err(&result, "Snapshot has already occurred");
}