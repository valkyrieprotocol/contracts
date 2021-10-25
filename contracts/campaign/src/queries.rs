use cosmwasm_std::{Deps, Env};

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::query_msgs::*;
use valkyrie::common::{ContractResult, Denom, OrderBy};
use valkyrie::utils::{compress_addr, put_query_parameter};

use crate::states::*;
use valkyrie::campaign_manager::query_msgs::ReferralRewardLimitOptionResponse;

pub fn get_campaign_config(deps: Deps, _env: Env) -> ContractResult<CampaignConfigResponse> {
    let campaign_config = CampaignConfig::load(deps.storage)?;

    Ok(CampaignConfigResponse {
        governance: campaign_config.governance.to_string(),
        campaign_manager: campaign_config.campaign_manager.to_string(),
        title: campaign_config.title,
        description: campaign_config.description,
        url: campaign_config.url,
        parameter_key: campaign_config.parameter_key,
        deposit_denom: campaign_config.deposit_denom.map(|d| Denom::from_cw20(d)),
        deposit_amount: campaign_config.deposit_amount,
        deposit_lock_period: campaign_config.deposit_lock_period,
        qualifier: campaign_config.qualifier.map(|e| e.to_string()),
        qualification_description: campaign_config.qualification_description,
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
        participation_reward_lock_period: reward_config.participation_reward_lock_period,
        referral_reward_token: reward_config.referral_reward_token.to_string(),
        referral_reward_amounts: reward_config.referral_reward_amounts,
        referral_reward_lock_period: reward_config.referral_reward_lock_period,
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
        deposit_amount: state.deposit_amount,
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

pub fn get_referral_reward_limit_amount(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<ReferralRewardLimitAmount> {
    let address = deps.api.addr_validate(address.as_str())?;

    let config = CampaignConfig::load(deps.storage)?;
    let option: ReferralRewardLimitOptionResponse = deps.querier.query_wasm_smart(
        &config.campaign_manager,
        &valkyrie::campaign_manager::query_msgs::QueryMsg::ReferralRewardLimitOption {},
    )?;

    let reward_config = RewardConfig::load(deps.storage)?;

    Ok(calc_referral_reward_limit(
        &option,
        &config,
        &reward_config,
        &deps.querier,
        &address,
    )?)
}

pub fn get_actor(
    deps: Deps,
    env: Env,
    address: String,
) -> ContractResult<ActorResponse> {
    let address = deps.api.addr_validate(&address)?;
    let actor = Actor::may_load(deps.storage, &address)?
        .unwrap_or_else(|| Actor::new(address.clone(), None));

    let (unlocked_participation_reward, locked_participation_reward) = actor.participation_reward_amount(env.block.height);
    let (unlocked_referral_reward, locked_referral_reward) = actor.referral_reward_amount(env.block.height);

    Ok(ActorResponse {
        address: actor.address.to_string(),
        referrer_address: actor.referrer.as_ref().map(|v| v.to_string()),
        participation_reward_amount: unlocked_participation_reward + locked_participation_reward,
        referral_reward_amount: unlocked_referral_reward + locked_referral_reward,
        participation_reward_amounts: actor.participation_reward_amounts,
        referral_reward_amounts: actor.referral_reward_amounts,
        cumulative_participation_reward_amount: actor.cumulative_participation_reward_amount,
        cumulative_referral_reward_amount: actor.cumulative_referral_reward_amount,
        participation_count: actor.participation_count,
        referral_count: actor.referral_count,
        last_participated_at: actor.last_participated_at,
    })
}

pub fn query_actors(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<ActorsResponse> {
    let start_after = start_after.map(|v| deps.api.addr_validate(&v)).transpose()?;
    let participations = Actor::query(deps.storage, start_after, limit, order_by)?
        .iter()
        .map(|actor| {
            let (unlocked_participation_reward, locked_participation_reward) = actor.participation_reward_amount(env.block.height);
            let (unlocked_referral_reward, locked_referral_reward) = actor.referral_reward_amount(env.block.height);

            ActorResponse {
                address: actor.address.to_string(),
                referrer_address: actor.referrer.as_ref().map(|v| v.to_string()),
                participation_reward_amount: unlocked_participation_reward + locked_participation_reward,
                referral_reward_amount: unlocked_referral_reward + locked_referral_reward,
                participation_reward_amounts: actor.participation_reward_amounts.clone(),
                referral_reward_amounts: actor.referral_reward_amounts.clone(),
                cumulative_participation_reward_amount: actor.cumulative_participation_reward_amount,
                cumulative_referral_reward_amount: actor.cumulative_referral_reward_amount,
                participation_count: actor.participation_count,
                referral_count: actor.referral_count,
                last_participated_at: actor.last_participated_at,
            }
        })
        .collect();

    Ok(ActorsResponse { actors: participations })
}

pub fn deposit(deps: Deps, _env: Env, address: String) -> ContractResult<Deposit> {
    Ok(Deposit::load_or_new(deps.storage, &deps.api.addr_validate(address.as_str())?)?)
}
