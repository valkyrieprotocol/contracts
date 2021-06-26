use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;

use valkyrie::campaign::execute_msgs::{ExecuteMsg, InstantiateMsg};
use valkyrie::campaign::query_msgs::QueryMsg;
use valkyrie::common::ContractResult;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
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
        ExecuteMsg::UpdateCampaignInfo {
            title,
            url,
            description,
        } => crate::executions::update_campaign_info(deps, env, info, title, url, description),
        ExecuteMsg::UpdateDistributionConfig { denom, amounts } => {
            crate::executions::update_distribution_config(deps, env, info, denom, amounts)
        }
        ExecuteMsg::UpdateAdmin { address } => {
            crate::executions::update_admin(deps, env, info, address)
        }
        ExecuteMsg::UpdateActivation { active } => {
            crate::executions::update_activation(deps, env, info, active)
        }
        ExecuteMsg::WithdrawReward { denom, amount } => {
            crate::executions::withdraw_reward(deps, env, info, denom, amount)
        }
        ExecuteMsg::ClaimReward {} => crate::executions::claim_reward(deps, env, info),
        ExecuteMsg::Participate { referrer } => {
            crate::executions::participate(deps, env, info, referrer)
        }
        ExecuteMsg::RegisterBooster {
            drop_booster_amount,
            activity_booster_amount,
            plus_booster_amount,
        } => crate::executions::register_booster(
            deps,
            env,
            info,
            drop_booster_amount,
            activity_booster_amount,
            plus_booster_amount,
        ),
        ExecuteMsg::DeregisterBooster {} => crate::executions::deregister_booster(deps, env, info),
    }
}

pub fn receive_cw20(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::CampaignInfo {} => to_binary(&crate::queries::get_campaign_info(deps, env)?),
        QueryMsg::DistributionConfig {} => {
            to_binary(&crate::queries::get_distribution_config(deps, env)?)
        }
        QueryMsg::CampaignState {} => to_binary(&crate::queries::get_campaign_state(deps, env)?),
        QueryMsg::BoosterState {} => to_binary(&crate::queries::get_booster_state(deps, env)?),
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
