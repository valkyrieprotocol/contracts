use cosmwasm_std::{Deps, Env};

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::query_msgs::*;
use valkyrie::common::{ContractResult, Denom, ExecutionMsg, OrderBy};
use valkyrie::utils::{compress_addr, put_query_parameter};

use crate::states::*;

pub fn get_campaign_config(deps: Deps, _env: Env) -> ContractResult<CampaignConfigResponse> {
    let campaign_config = CampaignConfig::load(deps.storage)?;

    Ok(CampaignConfigResponse {
        governance: campaign_config.governance.to_string(),
        campaign_manager: campaign_config.campaign_manager.to_string(),
        fund_manager: campaign_config.fund_manager.to_string(),
        title: campaign_config.title,
        description: campaign_config.description,
        url: campaign_config.url,
        parameter_key: campaign_config.parameter_key,
        qualifier: campaign_config.qualifier.map(|e| e.to_string()),
        executions: campaign_config.executions.iter()
            .map(|v| ExecutionMsg::from(v))
            .collect(),
        admin: campaign_config.admin.to_string(),
        creator: campaign_config.creator.to_string(),
        created_at: campaign_config.created_at,
    })
}

pub fn get_reward_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<RewardConfigResponse> {
    let reward_config = RewardConfig::load(deps.storage)?;

    Ok(RewardConfigResponse {
        participation_reward_denom: Denom::from_cw20(reward_config.participation_reward_denom),
        participation_reward_amount: reward_config.participation_reward_amount,
        referral_reward_token: reward_config.referral_reward_token.to_string(),
        referral_reward_amounts: reward_config.referral_reward_amounts,
    })
}

pub fn get_campaign_state(deps: Deps, env: Env) -> ContractResult<CampaignStateResponse> {
    let campaign_config = CampaignConfig::load(deps.storage)?;
    let state = CampaignState::load(deps.storage)?;

    Ok(CampaignStateResponse {
        actor_count: state.actor_count,
        participation_count: state.actor_count,
        cumulative_participation_reward_amount: state.cumulative_participation_reward_amount,
        cumulative_referral_reward_amount: state.cumulative_referral_reward_amount,
        locked_balances: state.locked_balances.iter()
            .map(|(denom, amount)| (Denom::from_cw20(denom.clone()), amount.clone()))
            .collect(),
        balances: state.balances.iter()
            .map(|(denom, amount)| (Denom::from_cw20(denom.clone()), amount.clone()))
            .collect(),
        is_active: state.is_active(& campaign_config, &deps.querier, &env.block)?,
        is_pending: state.is_pending(),
    })
}

pub fn get_share_url(deps: Deps, _env: Env, address: String) -> ContractResult<ShareUrlResponse> {
    deps.api.addr_validate(&address)?;

    let campaign_info = CampaignConfig::load(deps.storage)?;
    let compressed = compress_addr(&address);
    let url = put_query_parameter(
        &campaign_info.url,
        &campaign_info.parameter_key,
        &compressed,
    );

    Ok(ShareUrlResponse {
        address,
        compressed,
        url,
    })
}

pub fn get_address_from_referrer(
    deps: Deps,
    _env: Env,
    referrer: Referrer,
) -> ContractResult<GetAddressFromReferrerResponse> {
    Ok(GetAddressFromReferrerResponse {
        address: referrer.to_address(deps.api)?.to_string(),
    })
}

pub fn get_actor(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<ActorResponse> {
    let actor = Actor::load(
        deps.storage,
        &deps.api.addr_validate(&address)?,
    )?;

    Ok(ActorResponse {
        address: actor.address.to_string(),
        referrer_address: actor.referrer.as_ref().map(|v| v.to_string()),
        participation_reward_amount: actor.participation_reward_amount,
        referral_reward_amount: actor.referral_reward_amount,
        participation_count: actor.participation_count,
        referral_count: actor.referral_count,
        last_participated_at: actor.last_participated_at,
    })
}

pub fn query_participations(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<ActorsResponse> {
    let start_after = start_after.map(|v| deps.api.addr_validate(&v).unwrap());
    let participations = Actor::query(deps.storage, start_after, limit, order_by)?
        .iter()
        .map(|actor| {
            ActorResponse {
                address: actor.address.to_string(),
                referrer_address: actor.referrer.as_ref().map(|v| v.to_string()),
                participation_reward_amount: actor.participation_reward_amount,
                referral_reward_amount: actor.referral_reward_amount,
                participation_count: actor.participation_count,
                referral_count: actor.referral_count,
                last_participated_at: actor.last_participated_at,
            }
        })
        .collect();

    Ok(ActorsResponse { actors: participations })
}
