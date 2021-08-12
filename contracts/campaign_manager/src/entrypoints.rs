use cosmwasm_std::{Binary, Deps, DepsMut, Env, from_binary, MessageInfo, Reply, Response, StdError, to_binary};
use cosmwasm_std::entry_point;
use cw20::Cw20ReceiveMsg;

use valkyrie::campaign_manager::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::campaign_manager::query_msgs::QueryMsg;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::{executions, queries};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    crate::executions::instantiate(deps, env, info, msg)?;

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
            governance,
            fund_manager,
            terraswap_router,
            creation_fee_token,
            creation_fee_amount,
            creation_fee_recipient,
            code_id,
            withdraw_fee_rate,
            withdraw_fee_recipient,
            deactivate_period,
            key_denom,
            referral_reward_token,
            min_referral_reward_deposit_rate,
        } => executions::update_config(
            deps,
            env,
            info,
            governance,
            fund_manager,
            terraswap_router,
            creation_fee_token,
            creation_fee_amount,
            creation_fee_recipient,
            code_id,
            withdraw_fee_rate,
            withdraw_fee_recipient,
            deactivate_period,
            key_denom,
            referral_reward_token,
            min_referral_reward_deposit_rate,
        ),
        ExecuteMsg::UpdateReferralRewardLimitOption {
            overflow_amount_recipient,
            base_count,
            percent_for_governance_staking,
        } => executions::update_referral_reward_limit_option(
            deps,
            env,
            info,
            overflow_amount_recipient,
            base_count,
            percent_for_governance_staking,
        ),
        ExecuteMsg::SetReuseOverflowAmount {} => executions::set_reuse_overflow_amount(deps, env, info),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::CreateCampaign {
            config_msg,
            qualifier,
            executions,
        } => executions::create_campaign(
            deps,
            env,
            info,
            cw20_msg.sender,
            cw20_msg.amount,
            config_msg,
            qualifier,
            executions,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        crate::executions::REPLY_CREATE_CAMPAIGN => executions::created_campaign(deps, env, msg),
        _ => Err(ContractError::Std(StdError::not_found("reply_id")))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::Config {} => to_binary(
            &queries::get_config(deps, env)?
        ),
        QueryMsg::ReferralRewardLimitOption {} => to_binary(
            &queries::get_referral_reward_limit_option(deps, env)?
        ),
        QueryMsg::Campaign { address } => to_binary(
            &queries::get_campaign(deps, env, address)?
        ),
        QueryMsg::Campaigns {
            start_after,
            limit,
            order_by,
        } => to_binary(
            &queries::query_campaign(deps, env, start_after, limit, order_by)?
        ),
        QueryMsg::ReferralRewardLimitAmount {
            address,
        } => to_binary(
            &queries::get_referral_reward_limit_amount(deps, env, address)?
        ),
    }?;

    Ok(result)
}