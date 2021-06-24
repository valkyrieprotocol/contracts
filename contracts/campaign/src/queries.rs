use cosmwasm_std::{Deps, Env, Uint64, Uint128};

use valkyrie::campaign::enumerations::{Denom, Referrer};
use valkyrie::campaign::query_msgs::{CampaignInfoResponse, DistributionConfigResponse, CampaignStateResponse, ShareUrlResponse, GetAddressFromReferrerResponse, ParticipationResponse, ParticipationsResponse};
use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::utils::{map_uint128, find, compress_addr, put_query_parameter};

use crate::states::{CampaignInfo, DistributionConfig, CampaignState, Participation};
use valkyrie::cw20::query_balance;

pub fn get_campaign_info(
    deps: Deps,
    _env: Env,
) -> ContractResult<CampaignInfoResponse> {
    let campaign_info = CampaignInfo::load(deps.storage)?;

    Ok(CampaignInfoResponse {
        title: campaign_info.title,
        description: campaign_info.description,
        url: campaign_info.url,
        parameter_key: campaign_info.parameter_key,
        creator: campaign_info.creator.to_string(),
        created_at: campaign_info.created_at,
        created_block: Uint64::from(campaign_info.created_block),
    })
}

pub fn get_distribution_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<DistributionConfigResponse> {
    let distribution_config = DistributionConfig::load(deps.storage)?;

    Ok(DistributionConfigResponse {
        denom: Denom::from_cw20(distribution_config.denom),
        amounts: map_uint128(distribution_config.amounts),
    })
}

pub fn get_campaign_state(
    deps: Deps,
    env: Env,
) -> ContractResult<CampaignStateResponse> {
    let distribution_config = DistributionConfig::load(deps.storage)?;
    let state = CampaignState::load(deps.storage)?;

    let cumulative_distribution_amount = find(
        &state.cumulative_distribution_amount,
        |(denom, _)| { distribution_config.denom.eq(denom) },
    ).unwrap_or(&(distribution_config.denom.clone(), 0u128)).1;

    let locked_balance = find(
        &state.locked_balance,
        |(denom, _)| { distribution_config.denom.eq(denom) },
    ).unwrap_or(&(distribution_config.denom.clone(), 0u128)).1;

    let balance = query_balance(&deps.querier, deps.api, distribution_config.denom, env.contract.address)?;

    Ok(CampaignStateResponse {
        participation_count: Uint64::from(state.participation_count),
        cumulative_distribution_amount: Uint128::from(cumulative_distribution_amount),
        locked_balance: Uint128::from(locked_balance),
        balance: Uint128::from(balance),
        is_active: state.is_active(deps.storage, &deps.querier, env.block.height)?,
    })
}

pub fn get_share_url(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<ShareUrlResponse> {
    deps.api.addr_validate(&address)?;

    let campaign_info = CampaignInfo::load(deps.storage)?;
    let compressed = compress_addr(&address);
    let url = put_query_parameter(&campaign_info.url, &campaign_info.parameter_key, &compressed);

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

pub fn get_participation(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<ParticipationResponse> {
    let actor = deps.api.addr_validate(&address)?;
    let participation = Participation::load(deps.storage, &actor)?;
    let rewards: Vec<(Denom, Uint128)> = participation.rewards.iter()
        .map(|(denom, amount)| (Denom::from_cw20(denom.clone()), Uint128::from(amount.clone())))
        .collect();

    Ok(ParticipationResponse {
        actor_address: participation.actor_address.to_string(),
        referrer_address: participation.referrer_address.map(|v| v.to_string()),
        rewards,
    })
}

pub fn query_participations(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<ParticipationsResponse> {
    let start_after = start_after.map(|v| deps.api.addr_validate(&v).unwrap());
    let participations = Participation::query(deps.storage, start_after, limit, order_by)?
        .iter()
        .map(|v| {
            let rewards: Vec<(Denom, Uint128)> = v.rewards.iter()
                .map(|(denom, amount)| (Denom::from_cw20(denom.clone()), Uint128::from(amount.clone())))
                .collect();

            ParticipationResponse {
                actor_address: v.actor_address.to_string(),
                referrer_address: v.referrer_address.as_ref().map(|a| a.to_string()),
                rewards,
            }
        })
        .collect();

    Ok(ParticipationsResponse {
        participations,
    })
}