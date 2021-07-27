use std::cmp::max;

use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::cw20::create_send_msg_response;
use valkyrie::errors::ContractError;
use valkyrie::governance::execute_msgs::StakingConfigInitMsg;

use crate::common::states::{ContractConfig, load_available_balance};
use crate::staking::states::StakingConfig;

use super::states::{StakerState, StakingState};
use valkyrie::utils::make_response;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: StakingConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    StakingConfig {
        withdraw_delay: msg.withdraw_delay,
    }.save(deps.storage)?;

    StakingState {
        total_share: Uint128::zero(),
        unstaking_amount: Uint128::zero(),
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    withdraw_delay: Option<u64>,
) -> ContractResult<Response> {
    // Validate
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_config");

    let mut config = StakingConfig::load(deps.storage)?;

    if let Some(withdraw_delay) = withdraw_delay {
        config.withdraw_delay = withdraw_delay;
        response.add_attribute("is_updated_withdraw_delay", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn stake_governance_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance_token(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds sent")));
    }

    // Execute
    let mut response = make_response("stake_governance_token");

    let mut staking_state = StakingState::load(deps.storage)?;
    let mut staker_state = StakerState::load_safe(deps.storage, &sender)?;

    let contract_available_balance = load_available_balance(deps.as_ref())?
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

    response.add_attribute("sender", sender.as_str());
    response.add_attribute("share", share.to_string());
    response.add_attribute("amount", amount.to_string());

    Ok(response)
}

// Withdraw amount if not staked. By default all funds will be withdrawn.
pub fn unstake_governance_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let staker_state = StakerState::may_load(deps.storage, &info.sender)?;

    if staker_state.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Nothing staked")));
    }

    // Execute
    let mut response = make_response("unstake_governance_token");

    let staking_config = StakingConfig::load(deps.storage)?;
    let mut staker_state = staker_state.unwrap();
    let mut staking_state = StakingState::load(deps.storage)?;

    staker_state.clean_votes(deps.storage);

    let contract_available_balance = load_available_balance(deps.as_ref())?;
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

    let unlock_block = env.block.height + staking_config.withdraw_delay;

    staker_state.share = user_share.checked_sub(withdraw_share)?;
    staker_state.unstaking_amounts.push((unlock_block, withdraw_amount));
    staker_state.save(deps.storage)?;

    staking_state.total_share = total_share.checked_sub(withdraw_share)?;
    staking_state.unstaking_amount += withdraw_amount;
    staking_state.save(deps.storage)?;

    response.add_attribute("unstake_amount", withdraw_amount);
    response.add_attribute("unstake_share", withdraw_share);

    Ok(response)
}

pub fn withdraw_governance_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // Execute
    let mut staker_state = StakerState::load(deps.storage, &info.sender)?;

    let withdraw_amount = staker_state.withdraw_unstaked(deps.storage, env.block.height)
        .iter()
        .map(|(_, amount)| amount)
        .sum();

    staker_state.save(deps.storage)?;

    let mut staking_state = StakingState::load(deps.storage)?;
    staking_state.unstaking_amount = staking_state.unstaking_amount.checked_sub(withdraw_amount)?;
    staking_state.save(deps.storage)?;

    let contract_config = ContractConfig::load(deps.storage)?;
    Ok(create_send_msg_response(
        &contract_config.governance_token,
        &info.sender,
        withdraw_amount,
        "withdraw_governance_token",
    ))
}