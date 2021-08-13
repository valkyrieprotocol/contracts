use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, to_binary, from_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use valkyrie::common::ContractResult;
use valkyrie::fund_manager::execute_msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, Cw20HookMsg};
use valkyrie::fund_manager::query_msgs::QueryMsg;

use crate::{executions, queries};
use cw20::Cw20ReceiveMsg;

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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig {
            admins,
            terraswap_router,
            campaign_deposit_fee_burn_ratio,
            campaign_deposit_fee_recipient,
        } => executions::update_config(
            deps,
            env,
            info,
            admins,
            terraswap_router,
            campaign_deposit_fee_burn_ratio,
            campaign_deposit_fee_recipient,
        ),
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
        ExecuteMsg::Swap {
            denom,
            amount,
            route,
        } => executions::swap(deps, env, info, denom, amount, route),
        ExecuteMsg::DistributeCampaignDepositFee {
            amount,
        } => executions::distribute_campaign_deposit_fee(deps, env, info, amount),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::CampaignDepositFee {} => executions::receive_campaign_deposit_fee(
            deps,
            env,
            info,
            cw20_msg.amount,
        ),
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
