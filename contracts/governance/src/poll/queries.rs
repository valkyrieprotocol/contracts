use cosmwasm_std::{Addr, Deps, Env, StdError};

use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::errors::ContractError;
use valkyrie::governance::enumerations::PollStatus;
use valkyrie::governance::models::{PollConfigResponse, PollResponse, PollsResponse, PollStateResponse, VotersResponse, VotersResponseItem};

use crate::poll::state::Poll;

use super::state::{PollConfig, PollState};

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
            expiration_period: poll_config.expiration_period,
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
    let poll = match Poll::may_load(deps.storage, &poll_id)? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::Std(StdError::generic_err("Poll does not exist"))),
    }.unwrap();

    Ok(poll.to_response(deps.api)?)
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
        .map(|poll| poll.to_response(deps.api).unwrap())
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
    start_after: Option<Addr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<VotersResponse> {
    let poll = match Poll::may_load(deps.storage, &poll_id)? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::Std(StdError::generic_err("Poll does not exist"))),
    }.unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else {
        let after = if let Some(start_after) = start_after {
            Some(deps.api.addr_canonicalize(&start_after.as_str())?)
        } else {
            None
        };

        Poll::read_voters(
            deps.storage,
            &poll_id,
            after,
            limit,
            order_by,
        )?
    };

    let response_items = voters.iter().map(|(voter, voter_info)| {
        VotersResponseItem {
            voter: deps.api.addr_humanize(&voter).unwrap(),
            vote: voter_info.vote.clone(),
            balance: voter_info.balance,
        }
    }).collect();

    Ok(
        VotersResponse {
            voters: response_items,
        }
    )
}