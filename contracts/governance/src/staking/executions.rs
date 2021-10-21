use std::cmp::max;

use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128, SubMsg};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::common::states::{ContractConfig, load_available_balance};

use super::states::{StakerState, StakingState};
use valkyrie::utils::make_response;
use valkyrie::message_factories;
use valkyrie::governance::execute_msgs::{StakingConfigInitMsg, ExecuteMsg};
use crate::staking::states::StakingConfig;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: StakingConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    StakingConfig {
        distributor: msg.distributor.map(|d| deps.api.addr_validate(d.as_str())).transpose()?,
    }.save(deps.storage)?;

    StakingState {
        total_share: Uint128::zero(),
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_staking_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    distributor: Option<String>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_staking_config");

    let mut config = StakingConfig::load(deps.storage)?;

    if let Some(distributor) = distributor {
        config.distributor = Some(deps.api.addr_validate(distributor.as_str())?);
        response = response.add_attribute("is_updated_distributor", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn stake_governance_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> ContractResult<Response> {
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance_token(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds sent")));
    }

    let config = StakingConfig::load(deps.storage)?;

    let mut response = make_response("stake_governance_token");

    if let Some(distributor) = config.distributor {
        response.messages.push(SubMsg::new(message_factories::wasm_execute(
            &distributor,
            &valkyrie::distributor::execute_msgs::ExecuteMsg::Distribute {
                id: None,
            },
        )));
    }

    response.messages.push(SubMsg::new(message_factories::wasm_execute(
        &env.contract.address,
        &ExecuteMsg::StakeGovernanceTokenHook {
            staker: sender.to_string(),
            amount,
        },
    )));

    Ok(response)
}

pub fn stake_governance_token_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds sent")));
    }

    // Execute
    let mut response = make_response("stake_governance_token_hook");

    let sender = deps.api.addr_validate(staker.as_str())?;

    let mut staking_state = StakingState::load(deps.storage)?;
    let mut staker_state = StakerState::load_safe(deps.storage, &sender)?;

    let contract_available_balance = load_available_balance(deps.as_ref(), env.block.height)?
        .checked_sub(amount)?;

    let share = if contract_available_balance.is_zero() || staking_state.total_share.is_zero() {
        amount
    } else {
        amount.multiply_ratio(staking_state.total_share, contract_available_balance)
    };

    staking_state.total_share += share;
    staking_state.save(deps.storage)?;

    staker_state.share += share;
    staker_state.save(deps.storage)?;

    response = response.add_attribute("sender", sender.as_str());
    response = response.add_attribute("share", share.to_string());
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}

pub fn unstake_governance_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let config = StakingConfig::load(deps.storage)?;

    let mut response = make_response("stake_governance_token");

    if let Some(distributor) = config.distributor {
        response.messages.push(SubMsg::new(message_factories::wasm_execute(
            &distributor,
            &valkyrie::distributor::execute_msgs::ExecuteMsg::Distribute {
                id: None,
            },
        )));
    }

    response.messages.push(SubMsg::new(message_factories::wasm_execute(
        &env.contract.address,
        &ExecuteMsg::UnstakeGovernanceTokenHook {
            staker: info.sender.to_string(),
            amount,
        },
    )));

    Ok(response)
}

// Withdraw amount if not staked. By default all funds will be withdrawn.
pub fn unstake_governance_token_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: String,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let sender = deps.api.addr_validate(staker.as_str())?;

    let mut staker_state = StakerState::may_load(deps.storage, &sender)?
        .ok_or(ContractError::Std(StdError::generic_err("Nothing staked")))?;

    // Execute
    let mut response = make_response("unstake_governance_token_hook");

    let mut staking_state = StakingState::load(deps.storage)?;

    staker_state.clean_votes(deps.storage);

    let contract_available_balance = load_available_balance(deps.as_ref(), env.block.height)?;
    let total_share = staking_state.total_share;
    let locked_balance = staker_state.get_locked_balance();
    let locked_share = locked_balance.multiply_ratio(
        total_share,
        contract_available_balance,
    );
    let user_share = staker_state.share;
    let withdraw_share = amount.map(|v| {
        max(
            v.multiply_ratio(total_share, contract_available_balance),
            Uint128::new(1u128),
        )
    }).unwrap_or_else(|| user_share.checked_sub(locked_share).unwrap());
    let withdraw_amount = amount.unwrap_or_else(|| {
        withdraw_share.multiply_ratio(contract_available_balance, total_share)
    });

    if locked_share + withdraw_share > user_share {
        return Err(ContractError::Std(StdError::generic_err(
            "User is trying to unstake too many tokens.",
        )));
    }

    staker_state.share = user_share.checked_sub(withdraw_share)?;
    staker_state.save(deps.storage)?;

    staking_state.total_share = total_share.checked_sub(withdraw_share)?;
    staking_state.save(deps.storage)?;

    let contract_config = ContractConfig::load(deps.storage)?;
    response = response.add_message(message_factories::cw20_transfer(
        &contract_config.governance_token,
        &sender,
        withdraw_amount,
    ));

    response = response.add_attribute("unstake_amount", withdraw_amount);
    response = response.add_attribute("unstake_share", withdraw_share);

    Ok(response)
}
