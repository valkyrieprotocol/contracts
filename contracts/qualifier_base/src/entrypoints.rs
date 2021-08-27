use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Response, to_binary};

use crate::{executions, queries};
use crate::executions::ExecuteResult;
use crate::msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::errors::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    executions::instantiate(deps, env, info, msg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ExecuteResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            continue_option_on_fail,
        } => executions::update_config(deps, env, info, admin, continue_option_on_fail),
        ExecuteMsg::UpdateRequirement {
            min_token_balances,
            min_luna_staking,
            participation_limit,
        } => executions::update_requirement(
            deps,
            env,
            info,
            min_token_balances,
            min_luna_staking,
            participation_limit,
        ),
        ExecuteMsg::Qualify(msg) => executions::qualify(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let result = match msg {
        QueryMsg::Qualify(msg) => to_binary(&queries::qualify(deps, env, msg)?),
        QueryMsg::Requirement {} => to_binary(&queries::requirement(deps, env)?),
        QueryMsg::Config {} => to_binary(&queries::config(deps, env)?),
    }?;

    Ok(result)
}
