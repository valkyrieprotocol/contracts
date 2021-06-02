use valkyrie::governance::messages::{InstantiateMsg, Cw20HookMsg};
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult, Response, Decimal, Uint128, StdError, Storage, Api, Querier, from_binary};
use super::state::{Config, State};
use cw20::Cw20ReceiveMsg;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;

    let config = Config {
        valkyrie_token: CanonicalAddr::default(),
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: msg.expiration_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let state = State {
        contract_addr: deps.api.canonical_address(&env.contract.address)?,
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };

    Config::singleton(deps.storage).save(&config)?;
    State::singleton(deps.storage).save(&state)?;

    Ok(Response::default())
}

#[allow(clippy::too_many_arguments)]
pub fn update_governance_config(
    deps: DepsMut,
    env: Env,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    expiration_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> HandleResult {
    let api = deps.api;

    Config::singleton(deps.storage).update(|mut config| {
        if config.owner != api.canonical_address(&env.message.sender)? {
            return Err(StdError::unauthorized());
        }

        if let Some(owner) = owner {
            config.owner = api.canonical_address(&owner)?;
        }

        if let Some(quorum) = quorum {
            config.quorum = quorum;
        }

        if let Some(threshold) = threshold {
            config.threshold = threshold;
        }

        if let Some(voting_period) = voting_period {
            config.voting_period = voting_period;
        }

        if let Some(timelock_period) = timelock_period {
            config.timelock_period = timelock_period;
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