use crate::executions::{
    auto_stake, auto_stake_hook, bond, deposit_reward, unbond, withdraw_reward,
};
use crate::queries::query_staker_info;
use crate::states::{Config, State};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw20::Cw20ReceiveMsg;
use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::lp_staking::query_msgs::QueryMsg;
use valkyrie::utils::make_response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let response = make_response("instantiate");

    Config {
        token: deps.api.addr_validate(&msg.token.as_str())?,
        pair: deps.api.addr_validate(&msg.pair.as_str())?,
        lp_token: deps.api.addr_validate(&msg.lp_token.as_str())?,
    }
    .save(deps.storage)?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::Unbond { amount } => unbond(deps, info.sender, amount),
        ExecuteMsg::Withdraw {} => withdraw_reward(deps, info),

        ExecuteMsg::AutoStake {
            amount,
            slippage_tolerance,
        } => auto_stake(deps, env, info, amount, slippage_tolerance),
        ExecuteMsg::AutoStakeHook {
            staker_addr,
            prev_staking_token_amount,
        } => {
            let api = deps.api;
            auto_stake_hook(
                deps,
                env,
                info,
                api.addr_validate(&staker_addr)?,
                prev_staking_token_amount,
            )
        }
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            let config: Config = Config::load(deps.storage)?;

            // only staking token contract can execute this message
            if config.lp_token != info.sender {
                return Err(StdError::generic_err("unauthorized"));
            }

            let api = deps.api;
            bond(
                deps,
                api.addr_validate(cw20_msg.sender.as_str())?,
                cw20_msg.amount,
            )
        }
        Ok(Cw20HookMsg::DepositReward {}) => {
            let config: Config = Config::load(deps.storage)?;

            // only reward token contract can execute this message
            if config.token.as_str().to_string() != info.sender.as_str().to_string() {
                return Err(StdError::generic_err("unauthorized"));
            }

            deposit_reward(deps, cw20_msg.amount)
        }
        Err(_) => Err(StdError::generic_err("invalid cw20 hook message")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&Config::load(deps.storage)?),
        QueryMsg::State {} => to_binary(&State::load(deps.storage)?),
        QueryMsg::StakerInfo { staker_addr } => to_binary(&query_staker_info(deps, staker_addr)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
