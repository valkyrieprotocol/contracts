use std::cmp::max;

use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::common::states::{ContractConfig, load_available_balance};

use super::states::{StakerState, StakingState};
use valkyrie::utils::make_response;
use valkyrie::message_factories;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    StakingState {
        total_share: Uint128::zero(),
    }.save(deps.storage)?;

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
    _env: Env,
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

    staker_state.share = user_share.checked_sub(withdraw_share)?;
    staker_state.save(deps.storage)?;

    staking_state.total_share = total_share.checked_sub(withdraw_share)?;
    staking_state.save(deps.storage)?;

    let contract_config = ContractConfig::load(deps.storage)?;
    response.add_message(message_factories::cw20_transfer(
        &contract_config.governance_token,
        &info.sender,
        withdraw_amount,
    ));

    response.add_attribute("unstake_amount", withdraw_amount);
    response.add_attribute("unstake_share", withdraw_share);

    Ok(response)
}
