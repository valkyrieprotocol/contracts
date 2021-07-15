use cosmwasm_std::{Deps, Env};

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::query_msgs::{CampaignInfoResponse, CampaignStateResponse, DistributionConfigResponse, GetAddressFromReferrerResponse, ParticipationResponse, ParticipationsResponse, ShareUrlResponse, BoosterResponse, ContractConfigResponse, PrevBoostersResponse, ActiveBoosterResponse};
use valkyrie::common::{ContractResult, OrderBy, ExecutionMsg, Denom};
use valkyrie::utils::{compress_addr, put_query_parameter};

use crate::states::{CampaignInfo, CampaignState, DistributionConfig, Participation, BoosterState, ContractConfig, Booster};
use valkyrie::cw20::query_balance;

pub fn get_contract_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let config = ContractConfig::load(deps.storage)?;

    Ok(ContractConfigResponse {
        admin: config.admin.to_string(),
        governance: config.governance.to_string(),
        campaign_manager: config.campaign_manager.to_string(),
        fund_manager: config.fund_manager.to_string(),
        proxies: config.proxies.iter().map(|v| v.to_string()).collect(),
    })
}

pub fn get_campaign_info(deps: Deps, _env: Env) -> ContractResult<CampaignInfoResponse> {
    let campaign_info = CampaignInfo::load(deps.storage)?;

    Ok(CampaignInfoResponse {
        title: campaign_info.title,
        description: campaign_info.description,
        url: campaign_info.url,
        parameter_key: campaign_info.parameter_key,
        executions: campaign_info.executions.iter()
            .map(|v| ExecutionMsg::from(v))
            .collect(),
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
    let balance = query_balance(
        &deps.querier,
        deps.api,
        distribution_config.denom,
        env.contract.address,
    )?;

    Ok(CampaignStateResponse {
        participation_count: state.participation_count,
        cumulative_distribution_amount: state.cumulative_distribution_amount,
        locked_balance: state.locked_balance,
        balance,
        is_active: state.is_active(deps.storage, &deps.querier, &env.block)?,
        is_pending: state.is_pending(),
    })
}

pub fn get_active_booster(deps: Deps, _env: Env) -> ContractResult<ActiveBoosterResponse> {
    let booster = Booster::may_load_active(deps.storage)?
        .as_ref()
        .map(Booster::to_response);

    Ok(ActiveBoosterResponse {
        active_booster: booster,
    })
}

pub fn get_prev_booster(deps: Deps, _env: Env, booster_id: u64) -> ContractResult<BoosterResponse> {
    let booster = Booster::load_prev(deps.storage, booster_id)?;

    Ok(booster.to_response())
}

pub fn query_prev_boosters(
    deps: Deps,
    _env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<PrevBoostersResponse> {
    let result = Booster::query(
        deps.storage,
        start_after,
        limit,
        order_by,
    )?;

    Ok(PrevBoostersResponse {
        prev_boosters: result,
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
    let booster_state = BoosterState::load(deps.storage)?;
    let participation = Participation::load(
        deps.storage,
        &deps.api.addr_validate(&address)?,
    )?;

    Ok(ParticipationResponse {
        actor_address: participation.actor_address.to_string(),
        referrer_address: participation.referrer_address.as_ref().map(|v| v.to_string()),
        reward_amount: participation.reward_amount,
        drop_booster_amount: participation.calc_drop_booster_amount(
            deps.storage,
            booster_state.recent_booster_id,
        )?,
        activity_booster_amount: participation.activity_booster_reward_amount,
        plus_booster_amount: participation.plus_booster_reward_amount,
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
    let booster_state = BoosterState::load(deps.storage)?;
    let participations = Participation::query(deps.storage, start_after, limit, order_by)?
        .iter()
        .map(|v| {
            ParticipationResponse {
                actor_address: v.actor_address.to_string(),
                referrer_address: v.referrer_address.as_ref().map(|v| v.to_string()),
                reward_amount: v.reward_amount,
                drop_booster_amount: v.calc_drop_booster_amount(
                    deps.storage,
                    booster_state.recent_booster_id,
                ).unwrap(),
                activity_booster_amount: v.activity_booster_reward_amount,
                plus_booster_amount: v.plus_booster_reward_amount,
                participated_at: v.participated_at,
            }
        })
        .collect();

    Ok(ParticipationsResponse { participations })
}
