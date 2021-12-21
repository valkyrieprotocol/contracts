use cosmwasm_std::{Deps, Env, StdResult};

use valkyrie::common::ContractResult;
use valkyrie::governance::query_msgs::{TicketStateResponse};
use crate::vp::states::{compute_ticket, TicketConfig, TicketState};


pub fn get_ticket_config(
    deps: Deps,
) -> ContractResult<TicketConfig> {
    Ok(TicketConfig::load(deps.storage)?)
}

pub fn get_ticket_state(
    deps: Deps,
) -> ContractResult<TicketState> {
    Ok(TicketState::load_or_default(deps.storage))
}

pub fn get_ticket_staker_state(
    deps: Deps,
    env: Env,
    address: String,
) -> StdResult<TicketStateResponse> {
    let staker = deps.api.addr_validate(&address.as_str())?;

    let (_ticket_state, ticket_staker_state) = compute_ticket(&deps, &env, &staker)?;

    Ok(TicketStateResponse {
        address,
        reward_index: ticket_staker_state.reward_index,
        pending_reward: ticket_staker_state.pending_reward,
    })
}