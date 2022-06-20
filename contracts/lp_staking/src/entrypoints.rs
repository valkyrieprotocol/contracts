#[cfg(not(feature = "library"))]
use crate::executions::{
    bond, migrate_reward, unbond, update_config, withdraw,
};
use crate::queries::{query_config, query_staker_info, query_state};
use crate::states::{Config, State};
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Uint128, StdError};
use cw20::Cw20ReceiveMsg;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::lp_staking::query_msgs::QueryMsg;
use valkyrie::utils::{is_valid_schedule, make_response};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let response = make_response("instantiate");

    if !is_valid_schedule(&msg.distribution_schedule) {
        return Err(ContractError::Std(StdError::generic_err(
            "invalid schedule",
        )));
    }

    Config {
        admin: info.sender,
        token: deps.api.addr_validate(&msg.token.as_str())?,
        usdc_token: deps.api.addr_validate(&msg.token.as_str())?,
        pair: deps.api.addr_validate(&msg.pair.as_str())?,
        lp_token: deps.api.addr_validate(&msg.lp_token.as_str())?,
        whitelisted_contracts: msg.whitelisted_contracts.iter()
            .map(|item| deps.api.addr_validate(item.as_str()).unwrap())
            .collect(),
        distribution_schedule: msg.distribution_schedule,
    }
        .save(deps.storage)?;

    State {
        last_distributed: env.block.height,
        total_bond_amount: Uint128::zero(),
        global_reward_index: Decimal::zero(),
    }
        .save(deps.storage)?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::UpdateConfig {
            token,
            pair,
            lp_token,
            admin,
            whitelisted_contracts,
            distribution_schedule,
        } => update_config(deps, env, info, token, pair, lp_token, admin, whitelisted_contracts, distribution_schedule),
        ExecuteMsg::MigrateReward { recipient, amount } => {
            migrate_reward(deps, env, info, recipient, amount)
        }
        ExecuteMsg::ApproveAdminNominee {} => crate::executions::approve_admin_nominee(deps, env, info),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let config: Config = Config::load(deps.storage)?;

    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Bond {} => {
            // only staking token contract can execute this message
            if config.lp_token != deps.api.addr_validate(&info.sender.as_str())? {
                return Err(ContractError::Unauthorized {});
            }
            bond(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_height } => to_binary(&query_state(deps, block_height)?),
        QueryMsg::StakerInfo { staker } => to_binary(&query_staker_info(deps, env, staker)?),
    }?;

    Ok(result)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
