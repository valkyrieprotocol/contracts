use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, to_binary, from_binary, Addr};
use cosmwasm_std::entry_point;

use valkyrie::campaign::execute_msgs::{ExecuteMsg, MigrateMsg, Cw20HookMsg};
use valkyrie::campaign::query_msgs::QueryMsg;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::executions;
use cw20::Cw20ReceiveMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CampaignInstantiateMsg,
) -> ContractResult<Response> {
    let mut deps_mut = deps;

    crate::executions::instantiate(deps_mut.branch(), env, info, msg)?;

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
        ExecuteMsg::UpdateCampaignConfig {
            title,
            description,
            url,
            parameter_key,
            deposit_amount,
            deposit_lock_period,
            vp_burn_amount,
            qualifier,
            qualification_description,
            admin,
        } => crate::executions::update_campaign_config(
            deps,
            env,
            info,
            title,
            description,
            url,
            parameter_key,
            deposit_amount,
            deposit_lock_period,
            vp_burn_amount,
            qualifier,
            qualification_description,
            admin,
        ),
        ExecuteMsg::ApproveAdminNominee {} => crate::executions::approve_admin_nominee(deps, env, info),
        ExecuteMsg::UpdateRewardConfig {
            participation_reward_amount,
            participation_reward_distribution_schedule,
            referral_reward_amounts,
            referral_reward_lock_period,
        } => crate::executions::update_reward_config(
            deps,
            env,
            info,
            participation_reward_amount,
            participation_reward_distribution_schedule,
            referral_reward_amounts,
            referral_reward_lock_period,
        ),
        ExecuteMsg::SetNoQualification {} => crate::executions::set_no_qualification(
            deps,
            env,
            info,
        ),
        ExecuteMsg::UpdateActivation { active } => {
            crate::executions::update_activation(deps, env, info, active)
        }
        ExecuteMsg::AddRewardPool {
            participation_reward_amount,
            referral_reward_amount,
        } => crate::executions::add_reward_pool(
            deps,
            env,
            info,
            participation_reward_amount,
            referral_reward_amount,
        ),
        ExecuteMsg::RemoveRewardPool { denom, amount } => {
            crate::executions::remove_reward_pool(deps, env, info, denom, amount)
        }
        ExecuteMsg::ClaimParticipationReward {} => crate::executions::claim_participation_reward(deps, env, info),
        ExecuteMsg::ClaimReferralReward {} => crate::executions::claim_referral_reward(deps, env, info),
        ExecuteMsg::Participate { actor, referrer } => {
            crate::executions::participate(deps, env, info, actor, referrer)
        },
        ExecuteMsg::Deposit {} => {
            let sender = info.sender.clone();
            let funds = info.funds.iter()
                .map(|c| (cw20::Denom::Native(c.denom.clone()), c.amount))
                .collect();

            executions::deposit(deps, env, info, sender, funds)
        },
        ExecuteMsg::Withdraw {
            amount,
        } => executions::withdraw(deps, env, info, amount),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Deposit {} => {
            let sender = info.sender.clone();

            executions::deposit(
                deps,
                env,
                info,
                Addr::unchecked(cw20_msg.sender),
                vec![(cw20::Denom::Cw20(sender), cw20_msg.amount)],
            )
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> ContractResult<Response> {
    crate::migrations::migrate(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        executions::REPLY_QUALIFY_PARTICIPATION => executions::participate_qualify_result(deps, env, msg),
        _ => Err(ContractError::Std(StdError::not_found("reply_id")))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::CampaignConfig {} => to_binary(&crate::queries::get_campaign_config(deps, env)?),
        QueryMsg::RewardConfig {} => to_binary(&crate::queries::get_reward_config(deps, env)?),
        QueryMsg::CampaignState {} => to_binary(&crate::queries::get_campaign_state(deps, env)?),
        QueryMsg::ShareUrl { address } => {
            to_binary(&crate::queries::get_share_url(deps, env, address)?)
        }
        QueryMsg::GetAddressFromReferrer { referrer } => to_binary(
            &crate::queries::get_address_from_referrer(deps, env, referrer)?,
        ),
        QueryMsg::ReferralRewardLimitAmount { address } => to_binary(
            &crate::queries::get_referral_reward_limit_amount(deps, env, address)?,
        ),
        QueryMsg::Actor { address } => {
            to_binary(&crate::queries::get_actor(deps, env, address)?)
        }
        QueryMsg::Actors {
            start_after,
            limit,
            order_by,
        } => to_binary(&crate::queries::query_actors(
            deps,
            env,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::Deposit { address } => to_binary(&crate::queries::deposit(deps, env, address)?),
    }?;

    Ok(result)
}
