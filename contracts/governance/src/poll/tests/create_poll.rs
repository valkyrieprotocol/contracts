use cosmwasm_std::{Addr, attr, Env, MessageInfo, Response, to_binary, Uint128, StdResult, Binary};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ExecuteMsg;

use valkyrie::common::{ContractResult, Execution, ExecutionMsg};
use valkyrie::governance::enumerations::PollStatus;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::*;
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::poll::executions::create_poll;
use crate::poll::states::Poll;
use crate::tests::init_default;

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
    execution_msgs: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    deps.querier.plus_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(GOVERNANCE, &deposit_amount)],
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
    execution_msgs: Vec<ExecutionMsg>,
) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = mock_info(GOVERNANCE_TOKEN, &[]);

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
        vec![],
    )
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

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
        execution_msgs.clone(),
    );

    assert_eq!(response.attributes, vec![
        attr("action", "create_poll"),
        attr("creator", PROPOSER1),
        attr("poll_id", "1"),
        attr("end_height", (env.block.height + POLL_VOTING_PERIOD).to_string()),
    ]);

    let executions = execution_msgs.iter()
        .map(|e| Execution::from(&deps.api, e))
        .collect::<StdResult<Vec<Execution>>>().unwrap();
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
        executions,
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
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        vec![],
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_create_poll_invalid_deposit() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT.checked_sub(Uint128::new(1)).unwrap(),
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        vec![],
    );

    expect_generic_err(
        &result,
        format!("Must deposit more than {} token", POLL_PROPOSAL_DEPOSIT).as_str(),
    );
}

#[test]
fn failed_create_poll_invalid_title() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        "a".to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        vec![],
    );
    expect_generic_err(&result, "Title too short");

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        "0123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234".to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        vec![],
    );
    expect_generic_err(&result, "Title too long");
}

#[test]
fn failed_create_poll_invalid_description() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        "a".to_string(),
        None,
        vec![],
    );
    expect_generic_err(&result, "Description too short");

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        "0123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234".to_string(),
        None,
        vec![],
    );
    expect_generic_err(&result, "Description too long");
}

#[test]
fn failed_create_poll_invalid_link() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        Some("http://".to_string()),
        vec![],
    );
    expect_generic_err(&result, "Link too short");

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        Some("0123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234012345678901234567890123456789012345678901234567890123456789012340123456789012345678901234567890123456789012345678901234567890123401234567890123456789012345678901234567890123456789012345678901234".to_string()),
        vec![],
    );
    expect_generic_err(&result, "Link too long");
}

#[test]
fn failed_create_poll_transfer() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let executions = vec![
        ExecutionMsg {
            order: 0,
            contract: GOVERNANCE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                amount: Uint128::new(1),
                recipient: GOVERNANCE.to_string(),
            }).unwrap(),
        }
    ];

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        executions,
    );
    expect_generic_err(&result, "Can't use Transfer message");

    let executions = vec![
        ExecutionMsg {
            order: 0,
            contract: GOVERNANCE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                amount: Uint128::new(1),
                contract: GOVERNANCE.to_string(),
                msg: Binary::default(),
            }).unwrap(),
        }
    ];

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        executions,
    );
    expect_generic_err(&result, "Can't use Send message");

    let executions = vec![
        ExecutionMsg {
            order: 0,
            contract: GOVERNANCE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                amount: Uint128::new(1),
                spender: GOVERNANCE.to_string(),
                expires: None,
            }).unwrap(),
        }
    ];

    let result = exec(
        &mut deps,
        governance_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(PROPOSER1),
        POLL_PROPOSAL_DEPOSIT,
        POLL_TITLE.to_string(),
        POLL_DESCRIPTION.to_string(),
        None,
        executions,
    );
    expect_generic_err(&result, "Can't use IncreaseAllowance message");
}

pub fn mock_exec_msg(order: u64) -> ExecutionMsg {
    ExecutionMsg {
        order,
        contract: format!("Contract{}", order),
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: Uint128::new(1),
        }).unwrap(),
    }
}