use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use valkyrie::campaign::execute_msgs::{ExecuteMsg, MigrateMsg};
use valkyrie::campaign::query_msgs::QueryMsg;
use valkyrie::common::ContractResult;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use crate::executions;

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
        ExecuteMsg::UpdateContractConfig {
            admin,
            proxies,
        } => executions::update_contract_config(deps, env, info, admin, proxies),
        ExecuteMsg::UpdateCampaignInfo {
            title,
            description,
            url,
            parameter_key,
            executions,
        } => crate::executions::update_campaign_info(
            deps,
            env,
            info,
            title,
            description,
            url,
            parameter_key,
            executions,
        ),
        ExecuteMsg::UpdateDistributionConfig { denom, amounts } => {
            crate::executions::update_distribution_config(deps, env, info, denom, amounts)
        }
        ExecuteMsg::UpdateActivation { active } => {
            crate::executions::update_activation(deps, env, info, active)
        }
        ExecuteMsg::Withdraw { denom, amount } => {
            crate::executions::withdraw(deps, env, info, denom, amount)
        }
        ExecuteMsg::ClaimParticipationReward {} => crate::executions::claim_participation_reward(deps, env, info),
        ExecuteMsg::ClaimBoosterReward {} => crate::executions::claim_booster_reward(deps, env, info),
        ExecuteMsg::Participate { actor, referrer } => {
            crate::executions::participate(deps, env, info, actor, referrer)
        }
        ExecuteMsg::EnableBooster {
            drop_booster_amount,
            activity_booster_amount,
            plus_booster_amount,
            activity_booster_multiplier,
        } => crate::executions::enable_booster(
            deps,
            env,
            info,
            drop_booster_amount,
            activity_booster_amount,
            plus_booster_amount,
            activity_booster_multiplier,
        ),
        ExecuteMsg::DisableBooster {} => crate::executions::disable_booster(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::ContractConfig {} => to_binary(&crate::queries::get_contract_config(deps, env)?),
        QueryMsg::CampaignInfo {} => to_binary(&crate::queries::get_campaign_info(deps, env)?),
        QueryMsg::DistributionConfig {} => {
            to_binary(&crate::queries::get_distribution_config(deps, env)?)
        }
        QueryMsg::CampaignState {} => to_binary(&crate::queries::get_campaign_state(deps, env)?),
        QueryMsg::ActiveBooster {} => to_binary(&crate::queries::get_active_booster(deps, env)?),
        QueryMsg::PrevBooster { booster_id } => to_binary(
            &crate::queries::get_prev_booster(deps, env, booster_id)?
        ),
        QueryMsg::PrevBoosters {
            start_after,
            limit,
            order_by,
        } => to_binary(
            &crate::queries::query_prev_boosters(
                deps,
                env,
                start_after,
                limit,
                order_by
            )?
        ),
        QueryMsg::ShareUrl { address } => {
            to_binary(&crate::queries::get_share_url(deps, env, address)?)
        }
        QueryMsg::GetAddressFromReferrer { referrer } => to_binary(
            &crate::queries::get_address_from_referrer(deps, env, referrer)?,
        ),
        QueryMsg::Participation { address } => {
            to_binary(&crate::queries::get_participation(deps, env, address)?)
        }
        QueryMsg::Participations {
            start_after,
            limit,
            order_by,
        } => to_binary(&crate::queries::query_participations(
            deps,
            env,
            start_after,
            limit,
            order_by,
        )?),
    }?;

    Ok(result)
}
