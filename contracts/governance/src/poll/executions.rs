use cosmwasm_std::{Addr, Decimal, DepsMut, Env, from_binary, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128};
use cw20::Cw20ExecuteMsg;

use valkyrie::common::{ContractResult, Execution, ExecutionMsg};
use valkyrie::errors::ContractError;
use valkyrie::governance::enumerations::{PollStatus, VoteOption};
use valkyrie::governance::execute_msgs::{ExecuteMsg, PollConfigInitMsg};
use valkyrie::message_factories;
use valkyrie::utils::make_response;

use crate::common::states::{ContractConfig, load_available_balance};
use crate::poll::states::{PollExecutionContext, PollResult};
use crate::staking::states::StakerState;

use super::states::{get_poll_id, Poll, PollConfig, PollState};

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: PollConfigInitMsg,
) -> ContractResult<Response> {
    // Validate
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;

    // Execute
    let response = make_response("instantiate");

    let poll_config = PollConfig {
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        execution_delay_period: msg.execution_delay_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let poll_state = PollState {
        poll_count: 0,
        total_deposit: Uint128::zero(),
    };

    poll_config.save(deps.storage)?;
    poll_state.save(deps.storage)?;

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
pub fn update_poll_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    execution_delay_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_poll_config");

    let mut poll_config = PollConfig::load(deps.storage)?;

    if let Some(quorum) = quorum {
        validate_quorum(quorum)?;
        poll_config.quorum = quorum;
        response = response.add_attribute("is_updated_quorum", "true");
    }

    if let Some(threshold) = threshold {
        validate_threshold(threshold)?;
        poll_config.threshold = threshold;
        response = response.add_attribute("is_updated_threshold", "true");
    }

    if let Some(voting_period) = voting_period {
        poll_config.voting_period = voting_period;
        response = response.add_attribute("is_updated_voting_period", "true");
    }

    if let Some(execution_delay_period) = execution_delay_period {
        poll_config.execution_delay_period = execution_delay_period;
        response = response.add_attribute("is_updated_execution_delay_period", "true");
    }

    if let Some(proposal_deposit) = proposal_deposit {
        poll_config.proposal_deposit = proposal_deposit;
        response = response.add_attribute("is_updated_proposal_deposit", "true");
    }

    if let Some(period) = snapshot_period {
        poll_config.snapshot_period = period;
        response = response.add_attribute("is_updated_period", "true");
    }

    poll_config.save(deps.storage)?;

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
pub fn create_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposer: Addr,
    deposit_amount: Uint128,
    title: String,
    description: String,
    link: Option<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    // Validate
    validate_title(&title)?;
    validate_description(&description)?;
    validate_link(&link)?;
    validate_executions(&executions)?;

    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance_token(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let poll_config = PollConfig::load(deps.storage)?;
    if deposit_amount < poll_config.proposal_deposit {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Must deposit more than {} token", poll_config.proposal_deposit)
        )));
    }

    // Execute
    let mut response = make_response("create_poll");

    let executions = executions.iter()
        .map(|execution| Execution::from(deps.api, execution))
        .collect::<StdResult<Vec<Execution>>>()?;

    let mut poll = Poll {
        id: get_poll_id(deps.storage, &deposit_amount)?,
        creator: proposer.clone(),
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        abstain_votes: Uint128::zero(),
        end_height: env.block.height + poll_config.voting_period,
        title,
        description,
        link,
        executions,
        deposit_amount,
        total_balance_at_end_poll: None,
        snapped_staked_amount: None,
        _status: None,
    };

    poll.save_with_index(deps.storage)?;

    response = response.add_attribute("creator", proposer.as_str());
    response = response.add_attribute("poll_id", poll.id.to_string());
    response = response.add_attribute("end_height", poll.end_height.to_string());

    Ok(response)
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    option: VoteOption,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    let poll_state = PollState::load(deps.storage)?;

    if poll_id == 0 || poll_state.poll_count < poll_id {
        return Err(ContractError::Std(StdError::generic_err("Poll does not exist")));
    }

    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if !poll.in_progress(env.block.height) {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in progress")));
    }

    // Check the voter already has a vote on the poll
    if poll.is_voted(deps.storage, &info.sender) {
        return Err(ContractError::Std(StdError::generic_err("User has already voted.")));
    }

    let contract_available_balance = load_available_balance(deps.as_ref(), env.block.height)?;
    let mut staker_state = StakerState::load_safe(deps.storage, &info.sender)?;

    if !staker_state.can_vote(deps.storage, contract_available_balance, amount)? {
        return Err(ContractError::Std(StdError::generic_err("User does not have enough staked tokens.")));
    }

    // Execute
    let mut response = make_response("cast_vote");

    poll.vote(deps.storage, &mut staker_state, option.clone(), amount)?;
    poll.snapshot_staked_amount(deps.storage, env.block.height, contract_available_balance).ok(); //snapshot 실패하더라도 무시

    poll.save(deps.storage)?;
    staker_state.save(deps.storage)?;

    response = response.add_attribute("poll_id", &poll_id.to_string());
    response = response.add_attribute("amount", &amount.to_string());
    response = response.add_attribute("voter", info.sender.as_str());
    response = response.add_attribute("voter_option", option.to_string());

    Ok(response)
}

pub fn end_poll(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    // Validate
    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::InProgress {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in progress")));
    }

    if poll.end_height > env.block.height {
        return Err(ContractError::Std(StdError::generic_err("Voting period has not expired")));
    }

    // Execute
    let mut response = make_response("end_poll");

    let contract_config = ContractConfig::load(deps.storage)?;
    let mut poll_state = PollState::load(deps.storage)?;

    let (poll_result, staked_amount) = poll.get_result(deps.as_ref(), env.block.height)?;

    poll.status = if poll_result == PollResult::Passed {
        PollStatus::Passed
    } else {
        PollStatus::Rejected
    };

    // Refunds deposit only when quorum is reached
    if poll_result != PollResult::QuorumNotReached && !poll.deposit_amount.is_zero() {
        response = response.add_message(
            message_factories::cw20_transfer(
                &contract_config.governance_token,
                &poll.creator,
                poll.deposit_amount,
            )
        )
    }

    // Update poll status
    poll.total_balance_at_end_poll = Some(staked_amount);
    poll.save_with_index(deps.storage)?;

    // Decrease total deposit amount
    poll_state.total_deposit = poll_state.total_deposit.checked_sub(poll.deposit_amount)?;
    poll_state.save(deps.storage)?;

    response = response.add_attribute("poll_id", poll_id.to_string());
    response = response.add_attribute("result", poll_result.to_string());
    response = response.add_attribute("passed", (poll_result == PollResult::Passed).to_string());

    Ok(response)
}

pub const REPLY_EXECUTION: u64 = 1;

pub fn execute_poll(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    // Validate
    let poll_config = PollConfig::load(deps.storage)?;
    let poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::Passed {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in passed status")));
    }

    if poll.end_height + poll_config.execution_delay_period > env.block.height {
        return Err(ContractError::Std(StdError::generic_err("Execution delay period has not expired")));
    }

    let mut executions = poll.executions;
    if executions.is_empty() {
        return Err(ContractError::Std(StdError::generic_err("The poll does not have executions")));
    }

    // Execute
    let mut response = make_response("execute_poll");

    executions.sort();

    PollExecutionContext {
        poll_id: poll.id,
        execution_count: executions.len() as u64,
    }.save(deps.storage)?;

    response = response.add_submessage(SubMsg {
        id: REPLY_EXECUTION,
        msg: message_factories::wasm_execute(
            &env.contract.address,
            &ExecuteMsg::RunExecution {
                executions: executions.iter().map(|e| ExecutionMsg::from(e)).collect(),
            },
        ),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    });

    response = response.add_attribute("poll_id", poll_id.to_string());

    Ok(response)
}

pub fn run_execution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("run_execution");

    executions.sort_by_key(|e| e.order);

    for execution in executions.iter() {
        response = response.add_message(message_factories::wasm_execute_bin(
            &deps.api.addr_validate(&execution.contract)?,
            execution.msg.clone(),
        ));
    }

    response = response.add_attribute("execution_count", executions.len().to_string());

    Ok(response)
}

pub fn reply_execution(
    deps: DepsMut,
    _env: Env,
    msg: Reply,
) -> ContractResult<Response> {
    // Validate
    let poll_execution_context = PollExecutionContext::load(deps.storage)?;
    let mut poll = Poll::load(deps.storage, &poll_execution_context.poll_id)?;

    if poll.status == PollStatus::Failed || poll.status == PollStatus::Executed {
        return Err(ContractError::Std(StdError::generic_err("Already executed")));
    }

    // Execute
    let mut response = make_response("reply_execution");

    poll.status = if msg.result.is_ok() {
        PollStatus::Executed
    } else {
        PollStatus::Failed
    };

    poll.save_with_index(deps.storage)?;
    PollExecutionContext::clear(deps.storage);

    response = response.add_attribute("poll_status", poll.status.to_string());

    Ok(response)
}

pub fn snapshot_poll(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    // Validate
    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::InProgress {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in progress")));
    }

    // Execute
    let mut response = make_response("snapshot_poll");

    let contract_available_balance = load_available_balance(deps.as_ref(), env.block.height)?;
    let staked_amount = poll.snapshot_staked_amount(
        deps.storage,
        env.block.height,
        contract_available_balance,
    )?;
    poll.save(deps.storage)?;

    response = response.add_attribute("poll_id", poll_id.to_string());
    response = response.add_attribute("staked_amount", staked_amount);

    Ok(response)
}

// Validate_quorum returns an error if the quorum is invalid
/// (we require 0-1)
fn validate_quorum(quorum: Decimal) -> StdResult<()> {
    if quorum > Decimal::one() {
        Err(StdError::generic_err("quorum must be 0 to 1"))
    } else {
        Ok(())
    }
}

// Validate_threshold returns an error if the threshold is invalid
/// (we require 0-1)
fn validate_threshold(threshold: Decimal) -> StdResult<()> {
    if threshold > Decimal::one() {
        Err(StdError::generic_err("threshold must be 0 to 1"))
    } else {
        Ok(())
    }
}

// Validate_title returns an error if the title is invalid
fn validate_title(title: &str) -> StdResult<()> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(StdError::generic_err("Title too short"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(StdError::generic_err("Title too long"))
    } else {
        Ok(())
    }
}

// Validate_description returns an error if the description is invalid
fn validate_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Description too long"))
    } else {
        Ok(())
    }
}

// Validate_link returns an error if the link is invalid
fn validate_link(link: &Option<String>) -> StdResult<()> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(StdError::generic_err("Link too short"))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(StdError::generic_err("Link too long"))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn validate_executions(executions: &Vec<ExecutionMsg>) -> StdResult<()> {
    for execution in executions.iter() {
        match from_binary(&execution.msg) {
            Ok(Cw20ExecuteMsg::Transfer { amount: _, recipient: _ }) => {
                return Err(StdError::generic_err("Can't use Transfer message"))
            },
            Ok(Cw20ExecuteMsg::Send { amount: _, contract: _, msg: _ }) => {
                return Err(StdError::generic_err("Can't use Send message"))
            },
            Ok(Cw20ExecuteMsg::IncreaseAllowance { spender: _, amount: _, expires: _}) => {
                return Err(StdError::generic_err("Can't use IncreaseAllowance message"))
            }
            _ => continue
        }

    }

    Ok(())
}