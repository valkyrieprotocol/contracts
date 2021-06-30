use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, CosmosMsg, WasmMsg, Uint128, attr, to_binary};
use valkyrie::common::ContractResult;
use crate::poll::executions::end_poll;
use crate::tests::{default_env, default_info, init_default, env_set_height, TOKEN_CONTRACT, POLL_PROPOSAL_DEPOSIT, expect_generic_err, POLL_SNAPSHOT_PERIOD};
use cw20::Cw20ExecuteMsg;
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_info};
use crate::poll::states::{Poll, PollResult};
use valkyrie::governance::enumerations::{PollStatus, VoteOption};
use crate::poll::tests::cast_vote::{VOTER1, VOTER2, VOTER3};
use valkyrie::message_matchers;
use crate::poll::tests::create_poll::PROPOSER1;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, poll_id: u64) -> ContractResult<Response> {
    let response = end_poll(deps.as_mut(), env, info, poll_id)?;

    for msg in message_matchers::cw20_transfer(&response.messages) {
        deps.querier.minus_token_balances(&[(
            &msg.contract_addr,
            &[(MOCK_CONTRACT_ADDR, &msg.amount)],
        )]);
        deps.querier.plus_token_balances(&[(
            &msg.contract_addr,
            &[(&msg.recipient, &msg.amount)],
        )]);
    }

    Ok(response)
}

pub fn will_success(deps: &mut CustomDeps, poll_id: u64) -> (Env, MessageInfo, Response) {
    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let mut env = default_env();
    env_set_height(&mut env, poll.end_height + 1);

    let info = default_info();

    let response = exec(deps, env.clone(), info.clone(), poll_id).unwrap();

    (env, info, response)
}

#[test]
fn succeed_passed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128(100);
    let staker2_staked_amount = Uint128(100);
    let staker3_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(100));
    super::cast_vote::will_success(&mut deps, VOTER2, poll_id, VoteOption::No, Uint128(30));
    super::cast_vote::will_success(&mut deps, VOTER3, poll_id, VoteOption::Abstain, Uint128(100));

    will_success(&mut deps, poll_id);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Passed);
}

#[test]
fn succeed_rejected_threshold_not_reached() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128(100);
    let staker2_staked_amount = Uint128(100);
    let staker3_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(30));
    super::cast_vote::will_success(&mut deps, VOTER2, poll_id, VoteOption::No, Uint128(100));
    super::cast_vote::will_success(&mut deps, VOTER3, poll_id, VoteOption::Abstain, Uint128(10));

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert_eq!(response.messages, vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: PROPOSER1.to_string(),
                amount: POLL_PROPOSAL_DEPOSIT,
            }).unwrap(),
        })
    ]);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::ThresholdNotReached.to_string()),
        attr("passed", "false"),
    ]);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
}

#[test]
fn succeed_rejected_quorum_not_reached() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128(100);
    let staker2_staked_amount = Uint128(100);
    let staker3_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(1));

    let (_, _, response) = will_success(&mut deps, poll_id);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);
}

#[test]
fn succeed_rejected_zero_quorum() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let staker1_staked_amount = Uint128(100);
    let staker2_staked_amount = Uint128(100);
    let staker3_staked_amount = Uint128(100);

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, staker1_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER2, staker2_staked_amount);
    crate::staking::tests::stake::will_success(&mut deps, VOTER3, staker3_staked_amount);

    let poll_id = 1u64;

    let (_, _, response) = will_success(&mut deps, poll_id);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);
}

#[test]
fn succeed_end_poll_with_controlled_quorum() {}

#[test]
fn succeed_rejected_nothing_staked() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);

    let poll_id = 1u64;

    let (_, _, response) = will_success(&mut deps, poll_id);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);
}

#[test]
fn failed_before_end_height() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);

    let poll_id = 1u64;

    let result = exec(
        &mut deps,
        default_env(),
        default_info(),
        poll_id,
    );

    expect_generic_err(&result, "Voting period has not expired");
}

#[test]
fn failed_quorum_inflation_without_snapshot_poll() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake::will_success(&mut deps, VOTER1, Uint128(100));

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(100));

    crate::staking::tests::stake::will_success(&mut deps, VOTER2, Uint128(1000));

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let mut env = default_env();
    env_set_height(&mut env, poll.end_height - POLL_SNAPSHOT_PERIOD + 1);
    super::cast_vote::exec(
        &mut deps,
        env,
        mock_info(VOTER2, &[]),
        poll_id,
        VoteOption::Yes,
        Uint128(100),
    ).unwrap();

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert_eq!(response.attributes, vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("result", PollResult::QuorumNotReached.to_string()),
        attr("passed", "false"),
    ]);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    assert_eq!(poll.status, PollStatus::Rejected);
    assert_eq!(poll.snapped_staked_amount, Some(Uint128(1100)));
}
