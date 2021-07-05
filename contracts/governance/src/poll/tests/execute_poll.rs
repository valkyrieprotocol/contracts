use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Uint128, SubMsg, CosmosMsg, WasmMsg, ReplyOn};
use valkyrie::common::ContractResult;
use crate::poll::executions::{execute_poll, REPLY_EXECUTION};
use crate::tests::{ POLL_EXECUTION_DELAY_PERIOD, init_default, POLL_PROPOSAL_DEPOSIT};
use crate::poll::states::{Poll, PollExecutionContext};
use crate::poll::tests::cast_vote::VOTER1;
use valkyrie::governance::enumerations::VoteOption;
use crate::poll::tests::create_poll::{PROPOSER1, POLL_TITLE, POLL_DESCRIPTION, POLL_LINK, mock_exec_msg};
use valkyrie::test_utils::{default_sender, expect_generic_err, contract_env_height};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, poll_id: u64) -> ContractResult<Response> {
    execute_poll(deps.as_mut(), env, info, poll_id)
}

pub fn will_success(deps: &mut CustomDeps, poll_id: u64) -> (Env, MessageInfo, Response) {
    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let env = contract_env_height(poll.end_height + POLL_EXECUTION_DELAY_PERIOD);

    let info = default_sender();

    let response = exec(deps, env.clone(), info.clone(), poll_id).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let execution_msgs = vec![
        mock_exec_msg(2),
        mock_exec_msg(1),
        mock_exec_msg(3),
    ];

    super::create_poll::will_success(
        &mut deps,
        PROPOSER1,
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE,
        POLL_DESCRIPTION,
        Some(POLL_LINK),
        Some(execution_msgs.clone()),
    );
    crate::staking::tests::stake_governance_token::will_success(&mut deps, VOTER1, Uint128(100));

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(100));
    super::end_poll::will_success(&mut deps, poll_id);

    let sorted_execution_msgs = vec![
        mock_exec_msg(1),
        mock_exec_msg(2),
        mock_exec_msg(3),
    ];

    let sub_msgs: Vec<SubMsg> = sorted_execution_msgs.iter().map(|e| {
        SubMsg {
            id: REPLY_EXECUTION,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: e.contract.to_string(),
                msg: e.msg.clone(),
                send: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Always,
        }
    }).collect();

    let (_, _, response) = will_success(&mut deps, poll_id);
    assert_eq!(response.submessages, sub_msgs);

    let context = PollExecutionContext::load(&deps.storage).unwrap();
    assert_eq!(context, PollExecutionContext {
        poll_id,
        execution_count: execution_msgs.len() as u64,
    });
}

#[test]
fn failed_not_passed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let execution_msgs = vec![
        mock_exec_msg(2),
        mock_exec_msg(1),
        mock_exec_msg(3),
    ];

    super::create_poll::will_success(
        &mut deps,
        PROPOSER1,
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE,
        POLL_DESCRIPTION,
        Some(POLL_LINK),
        Some(execution_msgs),
    );
    crate::staking::tests::stake_governance_token::will_success(&mut deps, VOTER1, Uint128(100));

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::No, Uint128(100));
    super::end_poll::will_success(&mut deps, poll_id);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let env = contract_env_height(poll.end_height + POLL_EXECUTION_DELAY_PERIOD);

    let result = exec(
        &mut deps,
        env,
        default_sender(),
        poll_id,
    );

    expect_generic_err(&result, "Poll is not in passed status");
}

#[test]
fn failed_in_execution_delay() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let execution_msgs = vec![
        mock_exec_msg(2),
        mock_exec_msg(1),
        mock_exec_msg(3),
    ];

    super::create_poll::will_success(
        &mut deps,
        PROPOSER1,
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE,
        POLL_DESCRIPTION,
        Some(POLL_LINK),
        Some(execution_msgs),
    );
    crate::staking::tests::stake_governance_token::will_success(&mut deps, VOTER1, Uint128(100));

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(100));
    let (env, _, _ ) = super::end_poll::will_success(&mut deps, poll_id);

    let result = exec(
        &mut deps,
        env,
        default_sender(),
        poll_id,
    );

    expect_generic_err(&result, "Execution delay period has not expired");
}

#[test]
fn failed_empty_execution() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::create_poll::default(&mut deps);
    crate::staking::tests::stake_governance_token::will_success(&mut deps, VOTER1, Uint128(100));

    let poll_id = 1u64;

    super::cast_vote::will_success(&mut deps, VOTER1, poll_id, VoteOption::Yes, Uint128(100));
    super::end_poll::will_success(&mut deps, poll_id);

    let poll = Poll::load(&deps.storage, &poll_id).unwrap();
    let env = contract_env_height(poll.end_height + POLL_EXECUTION_DELAY_PERIOD);

    let result = exec(
        &mut deps,
        env,
        default_sender(),
        poll_id,
    );

    expect_generic_err(&result, "The poll does not have executions");
}