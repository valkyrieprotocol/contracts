use cosmwasm_std::{Binary, Deps, DepsMut, Env, from_binary, MessageInfo, Reply, Response, StdError, to_binary};
use cosmwasm_std::entry_point;
use cw20::Cw20ReceiveMsg;

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::factory::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use valkyrie::factory::query_msgs::QueryMsg;

use crate::states::is_token_contract;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let mut deps_mut = deps;

    crate::executions::instantiate(deps_mut.branch(), env.clone(), info.clone(), msg)?;

    Ok(Response::default())
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
        ExecuteMsg::UpdateConfig {
            campaign_code_id,
            creation_fee_amount,
        } => crate::executions::update_config(deps, env, info, campaign_code_id, creation_fee_amount),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    if !is_token_contract(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::CreateCampaign { init_msg }) => crate::executions::create_campaign(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            init_msg,
        ),
        Err(err) => Err(ContractError::Std(err)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        crate::executions::REPLY_CREATE_CAMPAIGN => crate::executions::created_campaign(deps, env, msg),
        _ => Err(ContractError::Std(StdError::not_found("reply_id")))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::Config {} => to_binary(
            &crate::queries::get_config(deps, env)?
        ),
        QueryMsg::Campaign { address } => to_binary(
            &crate::queries::get_campaign(deps, env, address)?
        )
    }?;

    Ok(result)
}