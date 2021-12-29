use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::community::execute_msgs::InstantiateMsg;
use valkyrie::message_factories;
use valkyrie::utils::make_response;

use crate::states::{Allowance, ContractConfig, ContractState};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    ContractConfig {
        admin: deps.api.addr_validate(msg.admin.as_str())?,
        managing_token: deps.api.addr_validate(msg.managing_token.as_str())?,
    }.save(deps.storage)?;

    ContractState {
        remain_allowance_amount: Uint128::zero(),
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
) -> ContractResult<Response> {
    // Validate
    let mut config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_config");

    if let Some(admin) = admin.as_ref() {
        ContractConfig::save_admin_nominee(deps.storage, &deps.api.addr_validate(admin)?)?;
        response = response.add_attribute("is_updated_admin_nominee", "true");
        config.admin = deps.api.addr_validate(admin.as_str())?;
        response = response.add_attribute("is_updated_admin", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn approve_admin_nominee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> ContractResult<Response> {
    // Validate
    let mut campaign_config = ContractConfig::load(deps.storage)?;
    if !campaign_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("approve_admin_nominee");

    let address = deps.api.addr_validate(address.as_str())?;
    if let Some(admin_nominee) = ContractConfig::may_load_admin_nominee(deps.storage)? {
        if admin_nominee != address {
            return Err(ContractError::Std(StdError::generic_err("It is not admin nominee")));
        }
    }

    campaign_config.admin = address;
    response = response.add_attribute("is_updated_admin", "true");

    campaign_config.save(deps.storage)?;

    Ok(response)
}

pub fn increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut state = ContractState::load(deps.storage)?;
    let free_balance = state.load_balance(&deps.querier, &env, &config.managing_token)?.free_balance;
    if free_balance < amount {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }

    // Execute
    let mut response = make_response("increase_allowance");

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load_or_default(deps.storage, &address)?;

    allowance.increase(amount.clone());
    allowance.save(deps.storage)?;

    state.remain_allowance_amount += amount;
    state.save(deps.storage)?;

    response = response.add_attribute("address", address.to_string());
    response = response.add_attribute("amount", amount.clone());
    response = response.add_attribute("allowed_amount", allowance.allowed_amount);
    response = response.add_attribute("remain_amount", allowance.remain_amount);

    Ok(response)
}

pub fn decrease_allowance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("decrease_allowance");

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load(deps.storage, &address)?;
    let mut state = ContractState::load(deps.storage)?;

    let amount = if let Some(amount) = amount {
        if allowance.remain_amount < amount {
            return Err(ContractError::Std(StdError::generic_err("Insufficient remain amount")));
        } else {
            amount
        }
    } else {
        allowance.remain_amount.clone()
    };

    allowance.decrease(amount)?;
    allowance.save_or_delete(deps.storage)?;

    state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
    state.save(deps.storage)?;

    response = response.add_attribute("address", address.to_string());
    response = response.add_attribute("amount", amount.to_string());
    response = response.add_attribute("allowed_amount", allowance.allowed_amount);
    response = response.add_attribute("remain_amount", allowance.remain_amount);

    Ok(response)
}

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    let mut state = ContractState::load(deps.storage)?;

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // Execute
    let mut response = make_response("transfer");

    let remain_amount = if config.is_admin(&info.sender) {
        let balance = state.load_balance(&deps.querier, &env, &config.managing_token)?;

        if balance.free_balance < amount {
            return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
        }

        balance.free_balance.checked_sub(amount)?
    } else {
        let allowance = Allowance::may_load(deps.storage, &info.sender)?;

        if let Some(mut allowance) = allowance {
            if allowance.remain_amount < amount {
                return Err(ContractError::ExceedLimit {});
            }

            allowance.remain_amount = allowance.remain_amount.checked_sub(amount)?;
            allowance.save_or_delete(deps.storage)?;

            state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
            state.save(deps.storage)?;

            allowance.remain_amount
        } else {
            return Err(ContractError::Unauthorized {});
        }
    };

    response = response.add_message(message_factories::wasm_execute(
        &config.managing_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_validate(&recipient)?.to_string(),
            amount,
        },
    ));

    response = response.add_attribute("requester", info.sender.as_str());
    response = response.add_attribute("recipient", recipient);
    response = response.add_attribute("amount", amount);
    response = response.add_attribute("remain_amount", remain_amount);

    Ok(response)
}
