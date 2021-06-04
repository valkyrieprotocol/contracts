// use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, QuerierWrapper, Response, StdError, StdResult, Uint128, WasmMsg,
};

use valkyrie::staking::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};

use crate::state::{read_staker_info, Config, StakerInfo, State, CONFIG, STAKER_INFO, STATE};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terra_cosmwasm::TerraQuerier;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use terraswap::querier::query_token_balance;

// #[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        valkyrie_token: deps.api.addr_canonicalize(&msg.valkyrie_token.as_str())?,
        liquidity_token: deps.api.addr_canonicalize(&msg.liquidity_token.as_str())?, //bond는 liquidity_token만 가능.
        pair_contract: deps.api.addr_canonicalize(&msg.pair_contract.as_str())?,
        distribution_schedule: msg.distribution_schedule,
    };

    CONFIG.save(deps.storage, &config)?;

    let state = State {
        last_distributed: env.block.height, //마지막 분배
        total_bond_amount: Uint128::zero(), //총 본딩 된 금액.
        global_reward_index: Decimal::zero(),
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response {
        messages: vec![],
        attributes: vec![],
        submessages: vec![],
        data: None,
    })
}

// #[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::AutoStake {
            token_amount,
            slippage_tolerance,
        } => auto_stake(deps, env, info, token_amount, slippage_tolerance),
        ExecuteMsg::AutoStakeHook {
            staker_addr,
            already_staked_amount,
        } => auto_stake_hook(deps, env, info, staker_addr, already_staked_amount),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Bond {} => {
            // only staking token contract can execute this message
            if config.liquidity_token != deps.api.addr_canonicalize(&info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }
            bond(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
    }
}

pub fn bond(deps: DepsMut, env: Env, sender_addr: String, amount: Uint128) -> StdResult<Response> {
    let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(&sender_addr.as_str())?;

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info: StakerInfo = read_staker_info(&deps.as_ref(), &sender_addr_raw)?;

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;

    // Increase bond_amount
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
    STAKER_INFO.save(deps.storage, &sender_addr_raw, &staker_info)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response {
        messages: vec![],
        attributes: vec![
            attr("action", "bond"),
            attr("owner", sender_addr),
            attr("amount", amount.to_string()),
        ],
        data: None,
        submessages: vec![],
    })
}

pub fn auto_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_amount: Uint128,
    slippage_tolerance: Option<Decimal>,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let token_addr = deps.api.addr_humanize(&config.valkyrie_token)?.to_string();
    let liquidity_token_addr = deps.api.addr_humanize(&config.liquidity_token)?.to_string();
    let pair_addr = deps.api.addr_humanize(&config.pair_contract)?.to_string();

    if info.funds.len() != 1 || info.funds[0].denom != "uusd".to_string() {
        return Err(StdError::generic_err("UST only."));
    }

    if info.funds[0].amount == Uint128::zero() {
        return Err(StdError::generic_err("Send UST more than zero."));
    }

    let uusd_amount: Uint128 = info.funds[0].amount;
    let already_staked_amount = query_token_balance(
        &deps.querier,
        deps.api,
        deps.api.addr_validate(liquidity_token_addr.as_str())?,
        env.contract.address.clone(),
    )?;

    let tax_amount: Uint128 = compute_uusd_tax(&deps.querier, uusd_amount)?;

    Ok(Response {
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
                            amount: (uusd_amount.clone().checked_sub(tax_amount))?,
                            info: AssetInfo::NativeToken {
                                denom: "uusd".to_string(),
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
                    denom: "uusd".to_string(),
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
        submessages: vec![],
    })
}

fn compute_uusd_tax(querier: &QuerierWrapper, amount: Uint128) -> StdResult<Uint128> {
    const DECIMAL_FRACTION: Uint128 = Uint128(1_000_000_000_000_000_000u128);
    let amount = amount;
    let terra_querier = TerraQuerier::new(querier);

    let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
    let tax_cap: Uint128 = (terra_querier.query_tax_cap("uusd".to_string())?).cap;
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

    let config: Config = CONFIG.load(deps.as_ref().storage)?;
    let liquidity_token = deps.api.addr_humanize(&config.liquidity_token)?;

    // stake all lp tokens received, compare with staking token amount before liquidity provision was executed
    let current_staking_token_amount = query_token_balance(
        &deps.querier,
        deps.api,
        liquidity_token,
        env.contract.address.clone(),
    )?;
    let amount_to_stake = (current_staking_token_amount.checked_sub(already_staked_amount))?;

    bond(deps, env, staker_addr, amount_to_stake)
}

pub fn unbond(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(&info.sender.as_str())?;

    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info: StakerInfo = read_staker_info(&deps.as_ref(), &sender_addr_raw)?;

    if staker_info.bond_amount < amount {
        return Err(StdError::generic_err("Cannot unbond more than bond amount"));
    }

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;

    // Decrease bond_amount
    state.total_bond_amount = (state.total_bond_amount.checked_sub(amount))?;
    STATE.save(deps.storage, &state)?;
    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    staker_info.bond_amount = (staker_info.bond_amount.checked_sub(amount))?;
    if staker_info.pending_reward.is_zero() && staker_info.bond_amount.is_zero() {
        //스테이킹된거 없고, 지급예정금액 없을때.
        STAKER_INFO.remove(deps.storage, sender_addr_raw.as_slice());
    } else {
        STAKER_INFO.save(deps.storage, sender_addr_raw.as_slice(), &staker_info)?;
    }

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.liquidity_token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.as_str().to_string(),
                amount,
            })?,
            send: vec![],
        })],
        attributes: vec![
            attr("action", "unbond"),
            attr("owner", info.sender.as_str()),
            attr("amount", amount.to_string()),
        ],
        data: None,
        submessages: vec![],
    })
}

// withdraw rewards to executor
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.as_str())?;

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = read_staker_info(&deps.as_ref(), &sender_addr_raw)?;

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.height);
    compute_staker_reward(&state, &mut staker_info)?;
    STATE.save(deps.storage, &state)?;

    //pending reward sender에게 전송.
    let amount = staker_info.pending_reward;
    staker_info.pending_reward = Uint128::zero();

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.bond_amount.is_zero() {
        STAKER_INFO.remove(deps.storage, sender_addr_raw.as_slice());
    } else {
        STAKER_INFO.save(deps.storage, sender_addr_raw.as_slice(), &staker_info)?;
    }

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.valkyrie_token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.as_str().to_string(),
                amount,
            })?,
            send: vec![],
        })],
        attributes: vec![
            attr("action", "withdraw"),
            attr("owner", info.sender.as_str()),
            attr("amount", amount.to_string()),
        ],
        data: None,
        submessages: vec![],
    })
}

// compute distributed rewards and update global reward index
fn compute_reward(config: &Config, state: &mut State, block_height: u64) {
    if state.total_bond_amount.is_zero() {
        state.last_distributed = block_height;
        return;
    }

    let mut distributed_amount: Uint128 = Uint128::zero();
    for s in config.distribution_schedule.iter() {
        //s.0 = 시작시점
        //s.1 = 종료시점
        if s.0 > block_height || s.1 < state.last_distributed {
            //현재위치가, 시작시점보다 이전이거나, 종료시점보다 크면 continue
            continue;
        }

        // min(s.1, block_height) - max(s.0, last_distributed)
        let passed_blocks =
            std::cmp::min(s.1, block_height) - std::cmp::max(s.0, state.last_distributed);
        //passed_blocks = (입력받은 height or 종료시점) - (마지막분배시점 or 시작시점)

        let num_blocks = s.1 - s.0;
        let distribution_amount_per_block: Decimal = Decimal::from_ratio(s.2, num_blocks);
        // distribution_amount_per_block = 이번회차 분배금액 / 블록의 갯수.
        //                               = 블록당 분배금액.
        distributed_amount += distribution_amount_per_block * Uint128(passed_blocks as u128);
        //분배금액의합 += 블록당분배금액 * 경과블록.
    }

    state.last_distributed = block_height;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_bond_amount);
    // state.global_reward_index = state.global_reward_index + (distributed_amount / state.total_bond_amount)
    // 누적 분배 비율
}

// withdraw reward to pending reward
fn compute_staker_reward(state: &State, staker_info: &mut StakerInfo) -> StdResult<()> {
    let pending_reward = (staker_info.bond_amount * state.global_reward_index)
        .checked_sub(staker_info.bond_amount * staker_info.reward_index)?;
    //  pending_reward = (본딩금액 * (global인덱스 - old인덱스)) 인덱스의 차이만큼..
    //  pending_reward = (본딩금액 * 이번회차%)

    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}

// #[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_height } => to_binary(&query_state(deps, block_height)?),
        QueryMsg::StakerInfo {
            staker,
            block_height,
        } => to_binary(&query_staker_info(deps, staker, block_height)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        valkyrie_token: deps.api.addr_humanize(&config.valkyrie_token)?.to_string(),
        staking_token: deps.api.addr_humanize(&config.liquidity_token)?.to_string(),
        distribution_schedule: config.distribution_schedule,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps, block_height: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = STATE.load(deps.storage)?;
    if let Some(block_height) = block_height {
        let config: Config = CONFIG.load(deps.storage)?;
        compute_reward(&config, &mut state, block_height);
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
    })
}

pub fn query_staker_info(
    deps: Deps,
    staker: String,
    block_height: Option<u64>,
) -> StdResult<StakerInfoResponse> {
    let staker_raw = deps.api.addr_canonicalize(&staker.as_str())?;

    let mut staker_info: StakerInfo = read_staker_info(&deps, &staker_raw)?;
    if let Some(block_height) = block_height {
        let config: Config = CONFIG.load(deps.storage)?;
        let mut state: State = STATE.load(deps.storage)?;

        compute_reward(&config, &mut state, block_height);
        compute_staker_reward(&state, &mut staker_info)?;
    }

    Ok(StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}

// #[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
