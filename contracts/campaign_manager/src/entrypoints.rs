use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, to_binary};
use cosmwasm_std::entry_point;
use cw2::{set_contract_version};

use valkyrie::campaign_manager::execute_msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::campaign_manager::query_msgs::QueryMsg;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::{executions, queries};

const CONTRACT_NAME: &str = "valkyrie-campaign-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::UpdateConfig {
            governance,
            valkyrie_token,
            vp_token,
            valkyrie_proxy,
            code_id,
            add_pool_fee_rate,
            add_pool_min_referral_reward_rate,
            remove_pool_fee_rate,
            fee_burn_ratio,
            fee_recipient,
            deactivate_period,
            key_denom,
            contract_admin,
        } => executions::update_config(
            deps,
            env,
            info,
            governance,
            valkyrie_token,
            vp_token,
            valkyrie_proxy,
            code_id,
            add_pool_fee_rate,
            add_pool_min_referral_reward_rate,
            remove_pool_fee_rate,
            fee_burn_ratio,
            fee_recipient,
            deactivate_period,
            key_denom,
            contract_admin,
        ),
        ExecuteMsg::ApproveContractAdminNominee {} => executions::approve_contract_admin_nominee(deps, env, info),
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
        ExecuteMsg::CreateCampaign {
            config_msg,
            deposit_denom,
            deposit_amount,
            deposit_lock_period,
            vp_burn_amount,
            qualifier,
            qualification_description,
        } => executions::create_campaign(
            deps,
            env,
            info,
            config_msg,
            deposit_denom,
            deposit_amount,
            deposit_lock_period,
            vp_burn_amount,
            qualifier,
            qualification_description,
        ),
        ExecuteMsg::SpendFee {
            amount,
        } => executions::spend_fee(deps, env, info, amount),
        ExecuteMsg::SwapFee {
            denom,
            amount,
            route,
        } => executions::swap_fee(deps, env, info, denom, amount, route),
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
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {

    if cw2::get_contract_version(deps.storage).is_err() {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, "1.0.8-beta.0".to_string())?;
    }

    //mig to v1.0.8-beta.1 to ??????
    // let info = cw2::get_contract_version(deps.storage)?;
    // if info.version == "1.0.8-beta.1" {
    //     migrations::??????::migrate(&deps, &env, &msg);
    //     set_contract_version(deps.storage, CONTRACT_NAME, "??????")
    // }

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
    }?;

    Ok(result)
}