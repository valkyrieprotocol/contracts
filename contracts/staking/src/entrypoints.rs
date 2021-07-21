use crate::staking::executions::{auto_stake, auto_stake_hook, bond, unbond, withdraw};
use crate::staking::queries::{query_config, query_staker_info, query_state};
use crate::staking::states::{Config, State, CONFIG, STATE};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw20::Cw20ReceiveMsg;
use valkyrie::staking::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        valkyrie_token: deps.api.addr_validate(&msg.valkyrie_token.as_str())?,
        liquidity_token: deps.api.addr_validate(&msg.liquidity_token.as_str())?,
        pair_contract: deps.api.addr_validate(&msg.pair_contract.as_str())?,
        distribution_schedule: msg.distribution_schedule,
    };

    CONFIG.save(deps.storage, &config)?;

    let state = State {
        last_distributed: env.block.height,
        total_bond_amount: Uint128::zero(),
        global_reward_index: Decimal::zero(),
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response {
        messages: vec![],
        attributes: vec![],
        data: None,
        events: vec![],
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
            if config.liquidity_token != deps.api.addr_validate(&info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }
            bond(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_height } => to_binary(&query_state(deps, block_height)?),
        QueryMsg::StakerInfo { staker } => to_binary(&query_staker_info(deps, env, staker)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
