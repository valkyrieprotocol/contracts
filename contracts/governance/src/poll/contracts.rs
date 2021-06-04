use cosmwasm_std::{Addr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, attr, CosmosMsg, WasmMsg, to_binary};
use valkyrie::governance::messages::{InstantiateMsg, PollConfigInitMsg};

use super::super::errors::ContractError;
use super::super::state::{Config, State};
use crate::common::state::ContractConfig;
use crate::staking::state::{StakingConfig, StakerState, StakingState};
use crate::poll::state::{PollConfig, PollState, Poll, get_poll_id, Execution};
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::governance::enumerations::{VoteOption, PollStatus};
use crate::cw20::load_cw20_balance;
use valkyrie::governance::models::{VoterInfo, ExecutionMsg};
use cw20::Cw20ExecuteMsg;

pub fn instantiate(
    deps: &DepsMut,
    _env: &Env,
    _info: &MessageInfo,
    msg: PollConfigInitMsg,
) -> ContractResult<Response> {
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;

    let poll_config = PollConfig {
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        execution_delay_period: msg.execution_delay_period,
        expiration_period: msg.expiration_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let poll_state = PollState {
        poll_count: 0,
    };

    poll_config.save(deps.storage)?;
    poll_state.save(deps.storage)?;

    Ok(Response::default())
}

#[allow(clippy::too_many_arguments)]
pub fn update_poll_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    execution_delay_period: Option<u64>,
    expiration_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> ContractResult<Response> {
    let sender_address = deps.api.addr_canonicalize(info.sender.as_str())?;

    if !ContractConfig::load(deps.storage)?.is_admin(sender_address) {
        return Err(ContractError::Unauthorized {})
    }

    PollConfig::singleton(deps.storage).update(|mut config| {
        if let Some(quorum) = quorum {
            config.quorum = quorum;
        }

        if let Some(threshold) = threshold {
            config.threshold = threshold;
        }

        if let Some(voting_period) = voting_period {
            config.voting_period = voting_period;
        }

        if let Some(execution_delay_period) = execution_delay_period {
            config.execution_delay_period = execution_delay_period;
        }

        if let Some(expiration_period) = expiration_period {
            config.expiration_period = expiration_period;
        }

        if let Some(proposal_deposit) = proposal_deposit {
            config.proposal_deposit = proposal_deposit;
        }

        if let Some(period) = snapshot_period {
            config.snapshot_period = period;
        }

        Ok(config)
    })?;

    Ok(Response::default())
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
    execution_msgs: Option<Vec<ExecutionMsg>>,
) -> ContractResult<Response> {
    validate_title(&title)?;
    validate_description(&description)?;
    validate_link(&link)?;

    let poll_config = PollConfig::load(deps.storage)?;
    if deposit_amount < poll_config.proposal_deposit {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Must deposit more than {} token", poll_config.proposal_deposit)
        )));
    }

    let mut executions_temp: Vec<Execution> = vec![];
    let executions = if let Some(execution_msgs) = execution_msgs {
        for msg in execution_msgs {
            let execution = Execution {
                order: msg.order,
                contract: deps.api.addr_canonicalize(msg.contract.as_str())?,
                msg: msg.msg,
            };
            executions_temp.push(execution)
        }
        Some(executions_temp)
    } else {
        None
    };

    let poll = Poll {
        id: get_poll_id(deps.storage, &deposit_amount)?,
        creator: deps.api.addr_canonicalize(proposer.as_str())?,
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        end_height: env.block.height + poll_config.voting_period,
        title,
        description,
        link,
        executions,
        deposit_amount,
        total_balance_at_end_poll: None,
        staked_amount: None,
    };

    poll.save(deps.storage)?;
    Poll::indexer_bucket(deps.storage, &PollStatus::InProgress)
        .save(&poll.id.to_be_bytes(), &true)?;

    Ok(
        Response {
            submessages: vec![],
            messages: vec![],
            attributes: vec![
                attr("action", "create_poll"),
                attr("creator", info.sender.as_str()),
                attr("poll_id", poll.id.to_string()),
                attr("end_height", poll.end_height),
            ],
            data: None,
        }
    )
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> ContractResult<Response> {
    let sender_address = deps.api.addr_canonicalize(info.sender.as_str())?;
    let contract_config = ContractConfig::load(deps.storage)?;
    let poll_state = PollState::load(deps.storage)?;

    if poll_id == 0 || poll_state.poll_count < poll_id {
        return Err(ContractError::Std(StdError::generic_err("Poll does not exist")));
    }

    let mut poll = Poll::load(deps.storage, &poll_id)?;
    if !poll.in_progress(env.block.height) {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in progress")));
    }

    // Check the voter already has a vote on the poll
    if poll.load_voter(deps.storage, &sender_address).is_ok() {
        return Err(ContractError::Std(StdError::generic_err("User has already voted.")));
    }

    let staking_state = StakingState::load(deps.storage)?;
    let mut staker_state = StakerState::may_load(deps.storage, &sender_address)?
        .unwrap_or(StakerState::default(&sender_address));

    // convert share to amount
    let total_share = staking_state.total_share;
    let contract_balance = load_cw20_balance(
        &deps.querier,
        &deps.api.addr_humanize(&contract_config.token_contract)?,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
    )?;
    let total_balance = contract_balance.checked_sub(staking_state.total_deposit)?;

    if staker_state.share.multiply_ratio(total_balance, total_share) < amount {
        return Err(ContractError::Std(StdError::generic_err("User does not have enough staked tokens.")));
    }

    // update tally info
    // TODO: Abstain 은??
    if vote == VoteOption::Yes {
        poll.yes_votes += amount;
    } else {
        poll.no_votes += amount;
    }

    let voter_info = VoterInfo {
        vote,
        balance: amount,
    };
    staker_state.locked_balance.push((poll_id, voter_info.clone()));
    staker_state.save(deps.storage)?;

    let poll_config = PollConfig::load(deps.storage)?;

    let time_to_end = poll.end_height - env.block.height;
    if time_to_end < poll_config.snapshot_period && poll.staked_amount.is_none() {
        poll.staked_amount = Some(total_balance);
    }

    poll.save(deps.storage)?;

    Ok(
        Response {
            submessages: vec![],
            messages: vec![],
            attributes: vec![
                attr("action", "cast_vote"),
                attr("poll_id", &poll_id.to_string()),
                attr("amount", &amount.to_string()),
                attr("voter", info.sender.as_str()),
                attr("voter_option", voter_info.vote),
            ],
            data: None,
        }
    )
}

pub fn end_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::InProgress {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in progress")));
    }

    if poll.end_height > env.block.height {
        return Err(ContractError::Std(StdError::generic_err("Voting period has not expired")));
    }

    let no = poll.no_votes.u128();
    let yes = poll.yes_votes.u128();

    let tallied_weight = yes + no;

    let mut poll_status = PollStatus::Rejected;
    let mut rejected_reason = "";
    let mut passed = false;

    let mut messages: Vec<CosmosMsg> = vec![];
    let contract_config = ContractConfig::load(deps.storage)?;
    let poll_config = PollConfig::load(deps.storage)?;
    let mut staking_state = StakingState::load(deps.storage)?;

    let (quorum, staked_weight) = if staking_state.total_share.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else if let Some(staked_amount) = poll.staked_amount {
        (Decimal::from_ratio(tallied_weight, staked_amount), staked_amount)
    } else {
        let contract_balance = load_cw20_balance(
            &deps.querier,
            &deps.api.addr_humanize(&contract_config.token_contract)?,
            &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        )?;
        let staked_weight = contract_balance.checked_sub(staking_state.total_deposit)?;

        (Decimal::from_ratio(tallied_weight, staked_weight), staked_weight)
    };

    if tallied_weight == 0 || quorum < poll_config.quorum {
        // Quorum: More than quorum of the total staked tokens at the end of the voting
        // period need to have participated in the vote.
        rejected_reason = "Quorum not reached";
    } else {
        if Decimal::from_ratio(yes, tallied_weight) > poll_config.threshold {
            //Threshold: More than 50% of the tokens that participated in the vote
            // (after excluding “Abstain” votes) need to have voted in favor of the proposal (“Yes”).
            poll_status = PollStatus::Passed;
            passed = true;
        } else {
            rejected_reason = "Threshold not reached";
        }

        // Refunds deposit only when quorum is reached
        if !poll.deposit_amount.is_zero() {
            messages.push(
                CosmosMsg.Wasm(WasmMsg::Execute {
                    contract_addr: deps.api.addr_humanize(&contract_config.token_contract)?.to_string(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: deps.api.addr_humanize(&poll.creator)?.to_string(),
                        amount: poll.deposit_amount,
                    })?,
                })
            )
        }
    }

    // Decrease total deposit amount
    staking_state.total_deposit = staking_state.total_deposit.checked_sub(poll.deposit_amount)?;
    staking_state.save(deps.storage)?;

    // Update poll indexer
    Poll::indexer_bucket(deps.storage, &PollStatus::InProgress).remove(&poll.id.to_be_bytes());
    Poll::indexer_bucket(deps.storage, &poll_status).save(&poll.id.to_be_bytes(), &true)?;

    // Update poll status
    poll.status = poll_status;
    poll.total_balance_at_end_poll = Some(staked_weight);
    poll.save(deps.storage)?;

    Ok(
        Response {
            submessages: vec![],
            messages,
            attributes: vec![
                attr("action", "end_poll"),
                attr("poll_id", poll_id.to_string()),
                attr("rejected_reason", rejected_reason),
                attr("passed", passed.to_string()),
            ],
            data: None,
        }
    )
}

pub fn execute_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    let poll_config = PollConfig::load(deps.storage)?;
    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::Passed {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in passed status")));
    }

    if poll.end_height + poll_config.execution_delay_period > env.block.height {
        return Err(ContractError::Std(StdError::generic_err("Timelock period has not expired")));
    }


    Poll::indexer_bucket(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes())?;
    Poll::indexer_bucket(deps.storage, &PollStatus::Executed)
        .save(&poll_id.to_be_bytes(), &true)?;

    poll.status = PollStatus::Executed;
    poll.save(deps.storage)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(all_executions) = poll.executions {
        let mut executions = all_executions;
        executions.sort();
        for execution in executions {
            messages.push(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps.api.addr_humanize(&execution.contract)?.to_string(),
                    msg: execution.msg,
                    send: vec![],
                })
            )
        }
    } else {
        return Err(ContractError::Std(StdError::generic_err("The poll does now have executions")));
    }

    Ok(
        Response {
            submessages: vec![],
            messages,
            attributes: vec![
                attr("action", "execute_poll"),
                attr("poll_id", poll_id.to_string()),
            ],
            data: None,
        }
    )
}

pub fn expire_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    let poll_config = PollConfig::load(deps.storage)?;
    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::Passed {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in passed status")));
    }

    if poll.executions.is_none() {
        return Err(ContractError::Std(StdError::generic_err(
            "Cannot make a text proposal to expired state",
        )));
    }

    if poll.end_height + poll_config.expiration_period > env.block.height {
        return Err(ContractError::Std(StdError::generic_err("Expire height has not been reached")));
    }

    Poll::indexer_bucket(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes())?;
    Poll::indexer_bucket(deps.storage, &PollStatus::Expired)
        .save(&poll_id.to_be_bytes(), &true)?;

    poll.status = PollStatus::Expired;
    poll.save(deps.storage)?;

    Ok(
        Response {
            submessages: vec![],
            messages: vec![],
            attributes: vec![
                attr("action", "expire_poll"),
                attr("poll_id", poll_id.to_string()),
            ],
            data: None,
        }
    )
}

pub fn snapshot_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> ContractResult<Response> {
    let poll_config = PollConfig::load(deps.storage)?;
    let mut poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::InProgress {
        return Err(ContractError::Std(StdError::generic_err("Poll is not in progress")));
    }

    let time_to_end = poll.end_height - env.block.height;
    if time_to_end > poll_config.snapshot_period {
        return Err(ContractError::Std(StdError::generic_err("Cannot snapshot at this height")));
    }

    if poll.staked_amount.is_some() {
        return Err(ContractError::Std(StdError::generic_err("Snapshot has already occurred")));
    }

    // store the current staked amount for quorum calculation
    let contract_config = ContractConfig::load(deps.storage)?;
    let staking_state = StakingState::load(deps.storage)?;
    let contract_balance = load_cw20_balance(
        &deps.querier,
        &deps.api.addr_humanize(&contract_config.token_contract)?,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
    )?;
    let staked_amount = contract_balance.checked_sub(staking_state.total_deposit)?;

    poll.staked_amount = Some(staked_amount);
    poll.save(deps.storage)?;

    Ok(
        Response {
            submessages: vec![],
            messages: vec![],
            attributes: vec![
                attr("action", "snapshot_poll"),
                attr("poll_id", poll_id.to_string()),
                attr("staked_amount", staked_amount),
            ],
            data: None,
        }
    )
}

/// validate_quorum returns an error if the quorum is invalid
/// (we require 0-1)
fn validate_quorum(quorum: Decimal) -> StdResult<()> {
    if quorum > Decimal::one() {
        Err(StdError::generic_err("quorum must be 0 to 1"))
    } else {
        Ok(())
    }
}

/// validate_threshold returns an error if the threshold is invalid
/// (we require 0-1)
fn validate_threshold(threshold: Decimal) -> StdResult<()> {
    if threshold > Decimal::one() {
        Err(StdError::generic_err("threshold must be 0 to 1"))
    } else {
        Ok(())
    }
}

/// validate_title returns an error if the title is invalid
fn validate_title(title: &str) -> StdResult<()> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(StdError::generic_err("Title too short"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(StdError::generic_err("Title too long"))
    } else {
        Ok(())
    }
}

/// validate_description returns an error if the description is invalid
fn validate_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Description too long"))
    } else {
        Ok(())
    }
}

/// validate_link returns an error if the link is invalid
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