use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use cw20::{Cw20ExecuteMsg};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::utils::{is_valid_schedule, make_response};
use valkyrie::governance::execute_msgs::{TicketConfigInitMsg};
use valkyrie::message_factories;
use crate::staking::states::StakerState;
use crate::vp::states::{compute_ticket, TicketConfig};

pub fn instantiate(
    deps: DepsMut,
    msg: TicketConfigInitMsg,
) -> ContractResult<Response> {
    let response = make_response("instantiate");

    if !is_valid_schedule(&msg.distribution_schedule) {
        return Err(ContractError::Std(StdError::generic_err(
            "invalid schedule",
        )));
    }

    TicketConfig {
        ticket_token: deps.api.addr_validate(msg.ticket_token.as_str())?,
        distribution_schedule: msg.distribution_schedule,
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_ticket_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ticket_token:Option<String>,
    distribution_schedule: Option<Vec<(u64, u64, Uint128)>>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_ticket_config");

    let mut config = TicketConfig::load(deps.storage)?;

    if let Some(ticket_token) = ticket_token {
        config.ticket_token = deps.api.addr_validate(ticket_token.as_str())?;
        response = response.add_attribute("is_updated_ticket_token", "true");
    }

    if let Some(distribution_schedule) = distribution_schedule {
        if !is_valid_schedule(&distribution_schedule) {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid schedule",
            )));
        }

        config.distribution_schedule = distribution_schedule;
        response = response.add_attribute("is_updated_distribution_schedule", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn ticket_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    let mut response = make_response("ticket_claim");

    let sender = info.sender;
    let (ticket_state, mut ticket_staker_state) = compute_ticket(&deps.as_ref(), &env, &sender)?;
    ticket_state.save(deps.storage)?;

    let amount = ticket_staker_state.pending_reward;

    let staker_state = StakerState::load_safe(deps.storage, &sender)?;
    if staker_state.share.is_zero() {
        ticket_staker_state.delete(deps.storage);
    } else {
        ticket_staker_state.pending_reward = Uint128::zero();
        ticket_staker_state.save(deps.storage)?;
    }

    let ticket_config = TicketConfig::load(deps.storage)?;
    response = response.add_message(message_factories::wasm_execute(
        &ticket_config.ticket_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: sender.to_string(),
            amount,
        },
    ));

    response = response.add_attribute("recipient", sender.to_string());
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}
