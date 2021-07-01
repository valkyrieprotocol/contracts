use cosmwasm_std::{Addr, attr, Env, MessageInfo, Response, to_binary, Uint128};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_info};
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::governance::enumerations::PollStatus;
use valkyrie::governance::models::ExecutionMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use crate::poll::executions::create_poll;
use crate::poll::states::{Execution, Poll};
use crate::tests::{init_default, POLL_PROPOSAL_DEPOSIT, POLL_VOTING_PERIOD, TOKEN_CONTRACT};
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_generic_err};

pub const PROPOSER1: &str = "Proposer1";

pub const POLL_TITLE: &str = "PollTitle";
pub const POLL_DESCRIPTION: &str = "PollDescription";
pub const POLL_LINK: &str = "https://poll.link";

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    proposer: Addr,
    deposit_amount: Uint128,
    title: String,
    description: String,
    link: Option<String>,
    execution_msgs: Option<Vec<ExecutionMsg>>,
) -> ContractResult<Response> {
    deps.querier.plus_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &deposit_amount)],
    )]);

    create_poll(
        deps.as_mut(),
        env,
        info,
        proposer,
        deposit_amount,
        title,
        description,
        link,
        execution_msgs,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    proposer: &str,
    deposit_amount: Uint128,
    title: &str,
    description: &str,
    link: Option<&str>,
    execution_msgs: Option<Vec<ExecutionMsg>>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(TOKEN_CONTRACT, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        Addr::unchecked(proposer),
        deposit_amount,
        title.to_string(),
        description.to_string(),
        link.map(|v| v.to_string()),
        execution_msgs,
    ).unwrap();

    (env, info, response)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    will_success(
        deps,
        PROPOSER1,
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE,
        POLL_DESCRIPTION,
        None,
        None,
    )
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

    let (env, _, response) = will_success(
        &mut deps,
        PROPOSER1,
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE,
        POLL_DESCRIPTION,
        Some(POLL_LINK),
        Some(execution_msgs.clone()),
    );

    assert_eq!(response.attributes, vec![
        attr("action", "create_poll"),
        attr("creator", PROPOSER1),
        attr("poll_id", "1"),
        attr("end_height", env.block.height + POLL_VOTING_PERIOD),
    ]);

    let executions = execution_msgs.iter()
        .map(|e| Execution::from(&deps.api, e))
        .collect();
    let poll = Poll::load(&deps.storage, &1).unwrap();
    assert_eq!(poll, Poll {
        id: 1,
        creator: Addr::unchecked(PROPOSER1),
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        abstain_votes: Uint128::zero(),
        end_height: env.block.height + POLL_VOTING_PERIOD,
        title: POLL_TITLE.to_string(),
        description: POLL_DESCRIPTION.to_string(),
        link: Some(POLL_LINK.to_string()),
        executions: Some(executions),
        deposit_amount: POLL_PROPOSAL_DEPOSIT,
        total_balance_at_end_poll: None,
        snapped_staked_amount: None,
        _status: Some(PollStatus::InProgress),
    });

    let polls = Poll::query(
        &deps.storage,
        Some(PollStatus::InProgress),
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(polls.len(), 1);
    assert_eq!(polls.first().unwrap().id, 1);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        None,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_create_poll_invalid_deposit() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT.checked_sub(Uint128(1)).unwrap(),
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        None,
    );

    expect_generic_err(
        &result,
        format!("Must deposit more than {} token", POLL_PROPOSAL_DEPOSIT).as_str(),
    );
}

#[test]
fn failed_create_poll_invalid_title() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        "a".to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        None,
    );
    expect_generic_err(&result, "Title too short");

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        "0123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234".to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        None,
    );
    expect_generic_err(&result, "Title too long");
}

#[test]
fn failed_create_poll_invalid_description() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        "a".to_string(),
        None,
        None,
    );
    expect_generic_err(&result, "Description too short");

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        "0123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234".to_string(),
        None,
        None,
    );
    expect_generic_err(&result, "Description too long");
}

#[test]
fn failed_create_poll_invalid_link() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        Some("http://".to_string()),
        None,
    );
    expect_generic_err(&result, "Link too short");

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        Some("0123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234".to_string()),
        None,
    );
    expect_generic_err(&result, "Link too long");
}

pub fn mock_exec_msg(order: u64) -> ExecutionMsg {
    ExecutionMsg {
        order,
        contract: format!("Contract{}", order),
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: Uint128(1),
        }).unwrap(),
    }
}