use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::states::{Config, StakerInfo, State};

use cw20::Cw20ExecuteMsg;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::{is_valid_schedule, make_response};

pub fn bond(
    deps: DepsMut,
    env: Env,
    sender_addr: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let mut response = make_response("bond");

    let sender_addr_raw: Addr = deps.api.addr_validate(&sender_addr.as_str())?;

    let config: Config = Config::load(deps.storage)?;

    if !config.is_authorized(&deps.as_ref(), &sender_addr_raw)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Can only called by wallet",
        )));
    }

    let mut state: State = State::load(deps.storage)?;
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
    state.compute_reward(&config, env.block.height);
    staker_info.compute_staker_reward(&state)?;

    // Increase bond_amount
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
    staker_info.save(deps.storage)?;
    state.save(deps.storage)?;

    response = response.add_attribute("owner", sender_addr);
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let config: Config = Config::load(deps.storage)?;
    let sender_addr_raw: Addr = info.sender;

    let mut state: State = State::load(deps.storage)?;
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &sender_addr_raw)?;

    if staker_info.bond_amount < amount {
        return Err(ContractError::Std(StdError::generic_err(
            "Cannot unbond more than bond amount",
        )));
    }

    let mut response = make_response("unbond");

    // Compute global reward & staker reward
    state.compute_reward(&config, env.block.height);
    staker_info.compute_staker_reward(&state)?;

    // Decrease bond_amount
    state.total_bond_amount = (state.total_bond_amount.checked_sub(amount))?;
    state.save(deps.storage)?;
    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    staker_info.bond_amount = (staker_info.bond_amount.checked_sub(amount))?;
    if staker_info.pending_reward.is_zero() && staker_info.bond_amount.is_zero() {
        //no bond, no reward.
        staker_info.delete(deps.storage);
    } else {
        staker_info.save(deps.storage)?;
    }

    response = response.add_message(message_factories::wasm_execute(
        &config.lp_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: sender_addr_raw.to_string(),
            amount,
        },
    ));

    response = response.add_attribute("owner", sender_addr_raw.to_string());
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}

// withdraw rewards to executor
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let mut response = make_response("withdraw");

    let sender_addr_raw = info.sender;

    let config: Config = Config::load(deps.storage)?;
    let mut state: State = State::load(deps.storage)?;
    let mut staker_info = StakerInfo::load_or_default(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
    state.compute_reward(&config, env.block.height);
    staker_info.compute_staker_reward(&state)?;
    state.save(deps.storage)?;

    let amount = staker_info.pending_reward;
    staker_info.pending_reward = Uint128::zero();

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.bond_amount.is_zero() {
        staker_info.delete(deps.storage);
    } else {
        staker_info.save(deps.storage)?;
    }

    response = response.add_message(message_factories::wasm_execute(
        &config.token,
        &Cw20ExecuteMsg::Transfer {
            recipient: sender_addr_raw.to_string(),
            amount,
        },
    ));

    response = response.add_attribute("owner", sender_addr_raw.to_string());
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token:Option<String>,
    pair:Option<String>,
    lp_token:Option<String>,
    admin:Option<String>,
    whitelisted_contracts: Option<Vec<String>>,
    distribution_schedule: Option<Vec<(u64, u64, Uint128)>>,
) -> ContractResult<Response> {
    let mut response = make_response("update_config");

    let mut config: Config = Config::load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(token) = token {
        config.token = deps.api.addr_validate(token.as_str())?;
        response = response.add_attribute("is_updated_token", "true");
    }

    if let Some(pair) = pair {
        config.pair = deps.api.addr_validate(pair.as_str())?;
        response = response.add_attribute("is_updated_pair", "true");
    }

    if let Some(lp_token) = lp_token {
        config.lp_token = deps.api.addr_validate(lp_token.as_str())?;
        response = response.add_attribute("is_updated_lp_token", "true");
    }

    if let Some(admin) = admin {
        config.admin = deps.api.addr_validate(admin.as_str())?;
        response = response.add_attribute("is_updated_admin", "true");
    }

    if let Some(whitelisted_contracts) = whitelisted_contracts {
        config.whitelisted_contracts = whitelisted_contracts.iter()
            .map(|item| deps.api.addr_validate(item.as_str()).unwrap())
            .collect();
        response = response.add_attribute("is_updated_whitelisted_contracts", "true");
    }

    if let Some(distribution_schedule) = distribution_schedule {

        let mut state = State::load(deps.storage)?;
        state.compute_reward(&config, env.block.height);
        state.save(deps.storage)?;

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

pub fn migrate_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let mut response = make_response("migrate_reward");

    let config = Config::load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    response = response.add_message(message_factories::wasm_execute(
        &config.token,
        &Cw20ExecuteMsg::Transfer {
            recipient: (&deps.api.addr_validate(recipient.as_str())?).to_string(),
            amount,
        },
    ));

    response = response.add_attribute("recipient", recipient.to_string());
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}