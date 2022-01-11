use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use valkyrie::common::ContractResult;
use valkyrie::community::execute_msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::community::query_msgs::QueryMsg;

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
            admin,
        } => executions::update_config(
            deps,
            env,
            info,
            admin,
        ),
        ExecuteMsg::ApproveAdminNominee {} => executions::approve_admin_nominee(deps, env, info),
        ExecuteMsg::IncreaseAllowance {
            address,
            amount,
        } => executions::increase_allowance(deps, env, info, address, amount),
        ExecuteMsg::DecreaseAllowance {
            address,
            amount,
        } => executions::decrease_allowance(deps, env, info, address, amount),
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
        QueryMsg::Balance {} => to_binary(&queries::get_balance(deps, env)?),
        QueryMsg::Allowance { address } => {
            to_binary(&queries::get_allowance(deps, env, address)?)
        }
        QueryMsg::Allowances {
            start_after,
            limit,
            order_by,
        } => to_binary(&queries::query_allowances(
            deps,
            env,
            start_after,
            limit,
            order_by,
        )?),
    }?;

    Ok(result)
}
