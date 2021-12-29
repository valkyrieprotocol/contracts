use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128, attr, SubMsg, Binary};
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::distributor::execute_msgs::InstantiateMsg;
use valkyrie::message_factories;
use valkyrie::utils::make_response;
use crate::states::{ContractConfig, Distribution, ContractState};
use valkyrie::cw20::query_cw20_balance;

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
        distribution_count: 0,
        locked_amount: Uint128::zero(),
        distributed_amount: Uint128::zero(),
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

pub fn register_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_height: u64,
    end_height: u64,
    recipient: String,
    amount: Uint128,
    message: Option<Binary>,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("register_distribution");
    let mut state = ContractState::load(deps.storage)?;

    state.distribution_count += 1;

    let distribution = Distribution {
        id: state.distribution_count,
        start_height,
        end_height,
        recipient: deps.api.addr_validate(recipient.as_str())?,
        amount,
        distributed_amount: Uint128::zero(),
        message,
    };
    response.attributes.push(attr("distribution_id", distribution.id.to_string()));

    distribution.save(deps.storage)?;

    let balance = query_cw20_balance(
        &deps.querier,
        &config.managing_token,
        &env.contract.address,
    )?;

    state.lock(balance, amount)?;
    state.save(deps.storage)?;

    Ok(response)
}

pub fn update_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    start_height: Option<u64>,
    end_height: Option<u64>,
    amount: Option<Uint128>,
    message: Option<Binary>,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_distribution");

    let mut distribution = Distribution::may_load(deps.storage, id)?
        .ok_or(StdError::not_found("Distribution"))?;

    let prev_released_amount = distribution.released_amount(env.block.height);

    if let Some(start_height) = start_height {
        if env.block.height >= distribution.start_height {
            return Err(ContractError::Std(StdError::generic_err("Can't modify start_height of started distribution")));
        }

        distribution.start_height = start_height;
        response = response.add_attribute("is_updated_start_height", "true");
    }

    if let Some(end_height) = end_height {
        distribution.end_height = end_height;
        response = response.add_attribute("is_updated_end_height", "true");
    }

    if distribution.end_height - distribution.start_height < 10000 {
        return Err(ContractError::Std(StdError::generic_err("Distribute period must be greater than 10000 block")));
    }

    if let Some(amount) = amount {
        if prev_released_amount > amount {
            return Err(ContractError::Std(StdError::generic_err("amount must be greater than released_amount")));
        }

        let mut state = ContractState::load(deps.storage)?;

        if distribution.amount > amount {
            state.unlock(distribution.amount.checked_sub(amount)?)?;
        } else {
            let balance = query_cw20_balance(
                &deps.querier,
                &config.managing_token,
                &env.contract.address,
            )?;

            state.lock(balance, amount.checked_sub(distribution.amount)?)?;
        }
        state.save(deps.storage)?;

        distribution.amount = amount;
        response = response.add_attribute("is_updated_amount", "true");
    }

    if let Some(message) = message {
        distribution.message = Some(message);
        response = response.add_attribute("is_updated_message", "true");
    }

    if prev_released_amount > distribution.released_amount(env.block.height) {
        return Err(ContractError::Std(StdError::generic_err("Can not decrease released_amount")));
    }

    distribution.save(deps.storage)?;

    Ok(response)
}

pub fn remove_distribution_message(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("remove_distribution_message");

    let mut distribution = Distribution::may_load(deps.storage, id)?
        .ok_or(StdError::not_found("Distribution"))?;

    distribution.message = None;
    response = response.add_attribute("is_updated_message", "true");

    distribution.save(deps.storage)?;

    Ok(response)
}

pub fn distribute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    id: Option<u64>,
) -> ContractResult<Response> {
    // Validate
    let mut distributions = if let Some(id) = id {
        vec![Distribution::may_load(deps.storage, id)?
            .ok_or(StdError::generic_err("This id is expired distribution or invalid id"))?]
    } else {
        Distribution::load_all(deps.storage)?
    };

    // Execute
    let mut response = make_response("distribute");

    if !distributions.is_empty() {
        let config = ContractConfig::load(deps.storage)?;
        let mut state = ContractState::load(deps.storage)?;

        for distribution in distributions.iter_mut() {
            let amount = distribution.released_amount(env.block.height)
                .checked_sub(distribution.distributed_amount)
                .unwrap_or(Uint128::zero());

            if amount.is_zero() {
                continue;
            }

            let send_msg = if let Some(message) = distribution.message.as_ref() {
                message_factories::wasm_execute(
                    &config.managing_token,
                    &Cw20ExecuteMsg::Send {
                        contract: distribution.recipient.to_string(),
                        amount,
                        msg: message.clone(),
                    }
                )
            } else {
                message_factories::cw20_transfer(
                    &config.managing_token,
                    &distribution.recipient,
                    amount,
                )
            };

            response.messages.push(SubMsg::new(send_msg));

            state.unlock(amount)?;
            state.distributed_amount += amount;
            distribution.distributed_amount += amount;

            if distribution.amount == distribution.distributed_amount {
                distribution.delete(deps.storage);
            } else {
                distribution.save(deps.storage)?;
            }

            response.attributes.push(attr("distribution", format!(
                "{}/{}/{}", distribution.id, distribution.recipient, distribution.amount,
            )));
        }

        state.save(deps.storage)?;
    }

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
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let state = ContractState::load(deps.storage)?;
    let balance = query_cw20_balance(
        &deps.querier,
        &config.managing_token,
        &env.contract.address,
    )?;
    let remain_amount = balance.checked_sub(state.locked_amount)?;

    if remain_amount < amount {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }

    // Execute
    let mut response = make_response("transfer");

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
