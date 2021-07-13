use cosmwasm_std::{
    attr, to_binary, Addr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, QuerierWrapper,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use crate::states::{Config, StakerInfo, State, UST};

use cw20::Cw20ExecuteMsg;
use terra_cosmwasm::TerraQuerier;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use terraswap::querier::query_token_balance;
use valkyrie::lp_staking::execute_msgs::ExecuteMsg;

pub fn bond(deps: DepsMut, env: Env, sender_addr: String, amount: Uint128) -> StdResult<Response> {
    let sender_addr_raw: Addr = deps.api.addr_validate(&sender_addr.as_str())?;

    let config: Config = Config::load(deps.storage)?;
    let mut state: State = State::load(deps.storage)?;
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
    state.compute_reward(&config,  env.block.height);
    staker_info.compute_staker_reward(&state)?;

    // Increase bond_amount
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
    staker_info.save(deps.storage)?;
    state.save(deps.storage)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "bond"),
            attr("owner", sender_addr),
            attr("amount", amount.to_string()),
        ],
        data: None,
    })
}

//CONTRACT: the executor must increase allowance of valkyrie token first before executing auto stake
pub fn auto_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_amount: Uint128,
    slippage_tolerance: Option<Decimal>,
) -> StdResult<Response> {
    let config: Config = Config::load(deps.storage)?;
    let token_addr = &config.token.as_str().to_string();
    let liquidity_token_addr = &config.lp_token.as_str().to_string();
    let pair_addr = &config.pair.as_str().to_string();

    if info.funds.len() != 1 || info.funds[0].denom != *UST {
        return Err(StdError::generic_err("UST only."));
    }

    if info.funds[0].amount == Uint128::zero() {
        return Err(StdError::generic_err("Send UST more than zero."));
    }

    let uusd_amount: Uint128 = info.funds[0].amount;
    let already_staked_amount = query_token_balance(
        &deps.querier,
        deps.api.addr_validate(liquidity_token_addr.as_str())?,
        env.contract.address.clone(),
    )?;

    let tax_amount: Uint128 = compute_uusd_tax(&deps.querier, uusd_amount)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![
            // 1. Transfer token asset to staking contract
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_addr.clone(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: token_amount,
                })?,
                send: vec![],
            }),
            // 2. Increase allowance of token for pair contract
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_addr.clone(),
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: pair_addr.to_string(),
                    amount: token_amount,
                    expires: None,
                })?,
                send: vec![],
            }),
            // 3. Provide liquidity
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_addr.to_string(),
                msg: to_binary(&PairExecuteMsg::ProvideLiquidity {
                    assets: [
                        Asset {
                            amount: (uusd_amount.checked_sub(tax_amount))?,
                            info: AssetInfo::NativeToken {
                                denom: UST.to_string(),
                            },
                        },
                        Asset {
                            amount: token_amount,
                            info: AssetInfo::Token {
                                contract_addr: deps.api.addr_validate(token_addr.as_str())?,
                            },
                        },
                    ],
                    slippage_tolerance,
                })?,
                send: vec![Coin {
                    denom: UST.to_string(),
                    amount: uusd_amount.checked_sub(tax_amount)?,
                }],
            }),
            // 4. Execute staking hook, will stake in the name of the sender
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::AutoStakeHook {
                    staker_addr: info.sender.to_string(),
                    already_staked_amount,
                })?,
                send: vec![],
            }),
        ],
        attributes: vec![
            attr("action", "auto_stake"),
            attr("tax_amount", tax_amount.to_string()),
        ],
        data: None,
    })
}

fn compute_uusd_tax(querier: &QuerierWrapper, amount: Uint128) -> StdResult<Uint128> {
    const DECIMAL_FRACTION: Uint128 = Uint128(1_000_000_000_000_000_000u128);
    let amount = amount;
    let terra_querier = TerraQuerier::new(querier);

    let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
    let tax_cap: Uint128 = (terra_querier.query_tax_cap(UST.to_string())?).cap;
    Ok(std::cmp::min(
        amount.checked_sub(amount.multiply_ratio(
            DECIMAL_FRACTION,
            DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
        ))?,
        tax_cap,
    ))
}

pub fn auto_stake_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker_addr: String,
    already_staked_amount: Uint128,
) -> StdResult<Response> {
    // only can be called by itself
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }

    let config: Config = Config::load(deps.as_ref().storage)?;
    let liquidity_token = config.lp_token;

    // stake all lp tokens received, compare with staking token amount before liquidity provision was executed
    let current_staking_token_amount =
        query_token_balance(&deps.querier, liquidity_token, env.contract.address.clone())?;
    let amount_to_stake = (current_staking_token_amount.checked_sub(already_staked_amount))?;

    bond(deps, env, staker_addr, amount_to_stake)
}

pub fn unbond(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let config: Config = Config::load(deps.storage)?;
    let sender_addr_raw: Addr = info.sender;

    let mut state: State = State::load(deps.storage)?;
    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &sender_addr_raw)?;

    if staker_info.bond_amount < amount {
        return Err(StdError::generic_err("Cannot unbond more than bond amount"));
    }

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

    Ok(Response {
        submessages: vec![],
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.lp_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender_addr_raw.to_string(),
                amount,
            })?,
            send: vec![],
        })],
        attributes: vec![
            attr("action", "unbond"),
            attr("owner", sender_addr_raw.to_string()),
            attr("amount", amount.to_string()),
        ],
        data: None,
    })
}

// withdraw rewards to executor
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
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

    Ok(Response {
        submessages: vec![],
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender_addr_raw.to_string(),
                amount,
            })?,
            send: vec![],
        })],
        attributes: vec![
            attr("action", "withdraw"),
            attr("owner", sender_addr_raw.to_string()),
            attr("amount", amount.to_string()),
        ],
        data: None,
    })
}
