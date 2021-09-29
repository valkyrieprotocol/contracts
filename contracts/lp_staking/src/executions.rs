use cosmwasm_std::{
    Addr, Coin, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use crate::states::{Config, StakerInfo, State, UST};

use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use terraswap::querier::query_token_balance;
use valkyrie::lp_staking::execute_msgs::ExecuteMsg;
use valkyrie::message_factories;
use valkyrie::utils::make_response;

pub fn bond(deps: DepsMut, staker_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let mut response = make_response("bond");

    let mut state: State = State::load(deps.storage)?;
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &staker_addr)?;

    // Withdraw reward to pending reward; before changing share
    before_share_change(&state, &mut staker_info)?;
    // Increase total short or bond amount
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;

    staker_info.save(deps.storage)?;
    state.save(deps.storage)?;

    response = response.add_attribute("staker_addr", staker_addr);
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}

pub fn auto_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    slippage_tolerance: Option<Decimal>,
) -> StdResult<Response> {
    let config: Config = Config::load(deps.storage)?;

    if amount == Uint128::zero() {
        return Err(StdError::generic_err("token amount > 0"));
    }

    if info.funds.len() != 1 {
        return Err(StdError::generic_err("must uusd only"));
    }

    if info.funds[0].denom != UST.to_string() {
        return Err(StdError::generic_err("must uusd"));
    }

    let native_asset: Asset = Asset {
        info: AssetInfo::NativeToken {
            denom: UST.to_string(),
        },
        amount: info.funds[0].amount,
    };

    let token_asset: Asset = Asset {
        info: AssetInfo::Token {
            contract_addr: config.token.as_str().to_string(),
        },
        amount: amount,
    };

    // get current lp token amount to later compute the recived amount
    let prev_staking_token_amount =
        query_token_balance(&deps.querier, config.lp_token, env.contract.address.clone())?;

    // compute tax
    let tax_amount: Uint128 = native_asset.compute_tax(&deps.querier)?;

    // 1. Transfer token asset to staking contract
    // 2. Increase allowance of token for pair contract
    // 3. Provide liquidity
    // 4. Execute staking hook, will stake in the name of the sender
    let mut response = make_response("auto_stake");

    // 1. Transfer token asset to staking contract
    response = response.add_message(message_factories::wasm_execute(
        &config.token,
        &Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: token_asset.amount,
        },
    ));

    // 2. Increase allowance of token for pair contract
    response = response.add_message(message_factories::wasm_execute(
        &config.token,
        &Cw20ExecuteMsg::IncreaseAllowance {
            spender: config.pair.as_str().to_string(),
            amount: token_asset.amount,
            expires: None,
        },
    ));

    // 3. Provide liquidity
    response = response.add_message(message_factories::wasm_execute_with_funds(
        &config.pair,
        vec![Coin {
            denom: native_asset.info.to_string(),
            amount: native_asset.amount.checked_sub(tax_amount)?,
        }],
        &PairExecuteMsg::ProvideLiquidity {
            assets: [
                Asset {
                    amount: native_asset.amount.checked_sub(tax_amount)?,
                    info: native_asset.info.clone(),
                },
                token_asset,
            ],
            slippage_tolerance,
            receiver: None,
        },
    ));

    // 4. Execute staking hook, will stake in the name of the sender
    response = response.add_message(message_factories::wasm_execute(
        &env.contract.address,
        &ExecuteMsg::AutoStakeHook {
            staker_addr: info.sender.to_string(),
            prev_staking_token_amount,
        },
    ));

    response = response.add_attribute("tax_amount", tax_amount.to_string());
    Ok(response)
}

pub fn auto_stake_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker_addr: Addr,
    prev_staking_token_amount: Uint128,
) -> StdResult<Response> {
    // only can be called by itself
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }
    let config: Config = Config::load(deps.storage)?;
    // stake all lp tokens received, compare with staking token amount before liquidity provision was executed
    let current_staking_token_amount =
        query_token_balance(&deps.querier, config.lp_token, env.contract.address)?;
    let amount_to_stake = current_staking_token_amount.checked_sub(prev_staking_token_amount)?;

    bond(deps, staker_addr, amount_to_stake)
}

pub fn unbond(deps: DepsMut, staker_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let config: Config = Config::load(deps.storage)?;
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &staker_addr)?;
    if staker_info.bond_amount < amount {
        return Err(StdError::generic_err("Cannot unbond more than bond amount"));
    }
    let mut state: State = State::load(deps.storage)?;
    // Distribute reward to pending reward; before changing share
    before_share_change(&state, &mut staker_info)?;

    // Decrease total short or bond amount
    state.total_bond_amount = state.total_bond_amount.checked_sub(amount)?;
    staker_info.bond_amount = staker_info.bond_amount.checked_sub(amount)?;

    // Update rewards info
    if staker_info.pending_reward.is_zero() && staker_info.bond_amount.is_zero() {
        staker_info.delete(deps.storage);
    } else {
        staker_info.save(deps.storage)?;
    }

    // Update pool info
    state.save(deps.storage)?;

    let mut response = make_response("unbond");
    response = response.add_message(message_factories::wasm_execute(
        &config.lp_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: staker_addr.to_string(),
            amount,
        },
    ));

    response = response.add_attribute("staker_addr", staker_addr);
    response = response.add_attribute("amount", amount.to_string());

    Ok(response)
}

// deposit_reward must be from reward token contract
pub fn deposit_reward(deps: DepsMut, reward: Uint128) -> StdResult<Response> {
    let mut state: State = State::load(deps.storage)?;

    if state.total_bond_amount.is_zero() {
        state.pending_reward += reward.clone();
    } else {
        let total_reward = reward.clone() + state.pending_reward;
        let total_reward_per_bond = Decimal::from_ratio(total_reward, state.total_bond_amount);
        state.global_reward_index = state.global_reward_index + total_reward_per_bond;
        state.pending_reward = Uint128::zero();
    }

    state.save(deps.storage)?;

    let mut response = make_response("deposit_reward");
    response = response.add_attribute("rewards", reward.to_string());
    Ok(response)
}

// withdraw all rewards or single reward depending on asset_token
pub fn withdraw_reward(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &info.sender)?;

    let state: State = State::load(deps.storage)?;
    // Withdraw reward to pending reward
    before_share_change(&state, &mut staker_info)?;
    let total_reward = staker_info.pending_reward;
    staker_info.pending_reward = Uint128::zero();
    // Update rewards info
    if staker_info.bond_amount.is_zero() {
        staker_info.delete(deps.storage);
    } else {
        staker_info.save(deps.storage)?;
    }

    let config: Config = Config::load(deps.storage)?;

    let mut response = make_response("withdraw");

    response = response.add_message(message_factories::wasm_execute(
        &config.token,
        &Cw20ExecuteMsg::Transfer {
            recipient: info.sender.as_str().to_string(),
            amount: total_reward,
        },
    ));

    response = response.add_attribute("owner", info.sender.as_str().to_string());
    response = response.add_attribute("amount", total_reward.to_string());

    Ok(response)
}

// withdraw reward to pending reward
pub fn before_share_change(state: &State, staker_info: &mut StakerInfo) -> StdResult<()> {
    let global_reward_index = state.global_reward_index;

    //pending = bond_amount * (global_reward_index - reward_index)
    let pending_reward = (staker_info.bond_amount * global_reward_index)
        .checked_sub(staker_info.bond_amount * staker_info.reward_index)?;

    staker_info.reward_index = global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}
