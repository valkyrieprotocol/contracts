use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use valkyrie::common::ContractResult;
use valkyrie::distributor::execute_msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::distributor::query_msgs::QueryMsg;

use crate::{executions, queries};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    executions::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            admins,
        } => executions::update_config(
            deps,
            env,
            info,
            admins,
        ),
        ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient,
            amount,
        } => executions::register_distribution(deps, env, info, start_height, end_height, recipient, amount),
        ExecuteMsg::UpdateDistribution {
            id,
            start_height,
            end_height,
            amount,
        } => executions::update_distribution(deps, env, info, id, start_height, end_height, amount),
        ExecuteMsg::Distribute {
            id,
        } => executions::distribute(deps, env, info, id),
        ExecuteMsg::Transfer {
            recipient,
            amount,
        } => executions::transfer(deps, env, info, recipient, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::Config {} => to_binary(&queries::get_config(deps, env)?),
        QueryMsg::State {} => to_binary(&queries::get_state(deps, env)?),
        QueryMsg::Distributions {} => to_binary(
            &queries::get_distributions(deps, env)?,
        ),
    }?;

    Ok(result)
}
