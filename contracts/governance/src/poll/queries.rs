use cosmwasm_std::{Deps, Env, StdError};

use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::errors::ContractError;
use valkyrie::governance::enumerations::PollStatus;
use valkyrie::governance::query_msgs::{PollConfigResponse, PollResponse, PollsResponse, PollStateResponse, VotersResponse};

use crate::poll::states::Poll;

use super::states::{PollConfig, PollState};
use valkyrie::governance::models::VoteInfoMsg;

pub fn get_poll_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<PollConfigResponse> {
    let poll_config = PollConfig::load(deps.storage)?;

    Ok(
        PollConfigResponse {
            quorum: poll_config.quorum,
            threshold: poll_config.threshold,
            voting_period: poll_config.voting_period,
            execution_delay_period: poll_config.execution_delay_period,
            proposal_deposit: poll_config.proposal_deposit,
            snapshot_period: poll_config.snapshot_period,
        }
    )
}

pub fn get_poll_state(
    deps: Deps,
    _env: Env,
) -> ContractResult<PollStateResponse> {
    let poll_state = PollState::load(deps.storage)?;

    Ok(
        PollStateResponse {
            poll_count: poll_state.poll_count,
            total_deposit: poll_state.total_deposit,
        }
    )
}

pub fn get_poll(
    deps: Deps,
    _env: Env,
    poll_id: u64,
) -> ContractResult<PollResponse> {
    let poll = Poll::may_load(deps.storage, &poll_id)?;

    if poll.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Poll does not exist")));
    }

    Ok(poll.unwrap().to_response())
}

pub fn query_polls(
    deps: Deps,
    _env: Env,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<PollsResponse> {
    let polls = Poll::query(deps.storage, filter, start_after, limit, order_by)?.iter()
        .map(|poll| poll.to_response())
        .collect();

    Ok(
        PollsResponse {
            polls
        }
    )
}

pub fn query_voters(
    deps: Deps,
    _env: Env,
    poll_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<VotersResponse> {
    let poll = Poll::may_load(deps.storage, &poll_id)?;

    if poll.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Poll does not exist")));
    }

    let start_after = start_after.map(|v| deps.api.addr_validate(&v).unwrap());

    let voters = if poll.unwrap().status != PollStatus::InProgress {
        vec![]
    } else {
        Poll::read_voters(deps.storage, &poll_id, start_after, limit, order_by)?
    };

    let response_items = voters.iter().map(|(voter, voter_info)| {
        VoteInfoMsg {
            voter: voter.to_string(),
            option: voter_info.option.clone(),
            amount: voter_info.amount,
        }
    }).collect();

    Ok(
        VotersResponse {
            voters: response_items,
        }
    )
}