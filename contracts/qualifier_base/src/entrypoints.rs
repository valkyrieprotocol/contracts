use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Response, to_binary};

use valkyrie_qualifier::query_msgs::QueryMsg;

use crate::{executions, queries};
use crate::executions::ExecuteResult;
use crate::msgs::{ExecuteMsg, InstantiateMsg};
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
        ExecuteMsg::UpdateAdmin {
            address,
        } => executions::update_admin(deps, env, info, address),
        ExecuteMsg::UpdateQualificationRequirement {
            continue_option_on_fail,
            min_token_balances,
            min_luna_staking,
        } => executions::update_qualification_requirement(
            deps,
            env,
            info,
            continue_option_on_fail,
            min_token_balances,
            min_luna_staking,
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
        QueryMsg::Qualify(msg) => to_binary(
            &queries::qualify(deps, env, msg)?
        ),
    }?;

    Ok(result)
}
