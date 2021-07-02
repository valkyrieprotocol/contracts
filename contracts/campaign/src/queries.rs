use cosmwasm_std::{Deps, Env, Uint128, Timestamp};

use valkyrie::campaign::enumerations::{Denom, Referrer};
use valkyrie::campaign::query_msgs::{CampaignInfoResponse, CampaignStateResponse, DistributionConfigResponse, GetAddressFromReferrerResponse, ParticipationResponse, ParticipationsResponse, ShareUrlResponse, BoosterStateResponse, BoosterResponse, ContractConfigResponse};
use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::utils::{compress_addr, find, put_query_parameter};

use crate::states::{CampaignInfo, CampaignState, DistributionConfig, Participation, BoosterState, ContractConfig};
use valkyrie::cw20::query_balance;

pub fn get_contract_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let config = ContractConfig::load(deps.storage)?;

    Ok(ContractConfigResponse {
        admin: config.admin.to_string(),
        governance: config.governance.to_string(),
        distributor: config.distributor.to_string(),
        token_contract: config.token_contract.to_string(),
        factory: config.factory.to_string(),
        burn_contract: config.burn_contract.to_string(),
    })
}

pub fn get_campaign_info(deps: Deps, _env: Env) -> ContractResult<CampaignInfoResponse> {
    let campaign_info = CampaignInfo::load(deps.storage)?;

    Ok(CampaignInfoResponse {
        title: campaign_info.title,
        description: campaign_info.description,
        url: campaign_info.url,
        parameter_key: campaign_info.parameter_key,
        creator: campaign_info.creator.to_string(),
        created_at: campaign_info.created_at,
    })
}

pub fn get_distribution_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<DistributionConfigResponse> {
    let distribution_config = DistributionConfig::load(deps.storage)?;

    Ok(DistributionConfigResponse {
        denom: Denom::from_cw20(distribution_config.denom),
        amounts: distribution_config.amounts,
    })
}

pub fn get_campaign_state(deps: Deps, env: Env) -> ContractResult<CampaignStateResponse> {
    let distribution_config = DistributionConfig::load(deps.storage)?;
    let state = CampaignState::load(deps.storage)?;

    let cumulative_distribution_amount =
        find(&state.cumulative_distribution_amount, |(denom, _)| {
            distribution_config.denom == *denom
        })
        .unwrap_or(&(distribution_config.denom.clone(), Uint128::zero()))
        .1;

    let locked_balance = find(&state.locked_balance, |(denom, _)| {
        distribution_config.denom == *denom
    })
    .unwrap_or(&(distribution_config.denom.clone(), Uint128::zero()))
    .1;

    let balance = query_balance(
        &deps.querier,
        deps.api,
        distribution_config.denom,
        env.contract.address,
    )?;

    Ok(CampaignStateResponse {
        participation_count: state.participation_count,
        cumulative_distribution_amount: Uint128::from(cumulative_distribution_amount),
        locked_balance: Uint128::from(locked_balance),
        balance,
        is_active: state.is_active(deps.storage, &deps.querier, env.block.height)?,
        is_pending: state.is_pending(),
    })
}

pub fn get_booster_state(deps: Deps, _env: Env) -> ContractResult<BoosterStateResponse> {
    let mut is_boosting = false;
    let mut assigned_total_amount = Uint128::zero();
    let mut snapped_participation_count = 0u64;
    let mut drop_booster: Option<BoosterResponse> = None;
    let mut activity_booster: Option<BoosterResponse> = None;
    let mut plus_booster: Option<BoosterResponse> = None;
    let mut boosted_at: Option<Timestamp> = None;

    let booster = BoosterState::may_load(deps.storage)?;

    if let Some(booster) = booster {
        is_boosting = true;
        assigned_total_amount = booster.drop_booster_amount + booster.activity_booster_amount + booster.plus_booster_amount;
        snapped_participation_count = booster.drop_booster_participations;
        drop_booster = Some(BoosterResponse {
            assigned_amount: booster.drop_booster_amount,
            distributed_amount: booster.drop_booster_amount.checked_sub(booster.drop_booster_left_amount)?,
        });
        activity_booster = Some(BoosterResponse {
            assigned_amount: booster.activity_booster_amount,
            distributed_amount: booster.activity_booster_amount.checked_sub(booster.activity_booster_left_amount)?,
        });
        plus_booster = Some(BoosterResponse {
            assigned_amount: booster.plus_booster_amount,
            distributed_amount: booster.plus_booster_amount.checked_sub(booster.plus_booster_left_amount)?,
        });
        boosted_at = Some(booster.boosted_at);
    }

    Ok(BoosterStateResponse {
        is_boosting,
        assigned_total_amount,
        snapped_participation_count,
        drop_booster,
        activity_booster,
        plus_booster,
        boosted_at,
    })
}

pub fn get_share_url(deps: Deps, _env: Env, address: String) -> ContractResult<ShareUrlResponse> {
    deps.api.addr_validate(&address)?;

    let campaign_info = CampaignInfo::load(deps.storage)?;
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

pub fn get_participation(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<ParticipationResponse> {
    let actor = deps.api.addr_validate(&address)?;
    let participation = Participation::load(deps.storage, &actor)?;
    let rewards: Vec<(Denom, Uint128)> = participation
        .rewards
        .iter()
        .map(|(denom, amount)| (Denom::from_cw20(denom.clone()), *amount))
        .collect();

    Ok(ParticipationResponse {
        actor_address: participation.actor_address.to_string(),
        referrer_address: participation.referrer_address.map(|v| v.to_string()),
        rewards,
        participated_at: participation.participated_at,
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
            let rewards: Vec<(Denom, Uint128)> = v
                .rewards
                .iter()
                .map(|(denom, amount)| (Denom::from_cw20(denom.clone()), *amount))
                .collect();

            ParticipationResponse {
                actor_address: v.actor_address.to_string(),
                referrer_address: v.referrer_address.as_ref().map(|a| a.to_string()),
                rewards,
                participated_at: v.participated_at,
            }
        })
        .collect();

    Ok(ParticipationsResponse { participations })
}
