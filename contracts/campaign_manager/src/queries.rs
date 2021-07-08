use cosmwasm_std::{Deps, Env};
use valkyrie::common::{ContractResult, Denom, OrderBy};
use crate::states::{ContractConfig, CampaignConfig, BoosterConfig, Campaign};
use valkyrie::campaign_manager::query_msgs::{ContractConfigResponse, CampaignConfigResponse, BoosterConfigResponse, CampaignResponse, CampaignsResponse};

pub fn get_contract_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let config = ContractConfig::load(deps.storage)?;

    Ok(ContractConfigResponse{
        governance: config.governance.to_string(),
        fund_manager: config.fund_manager.to_string(),
    })
}

pub fn get_campaign_config(deps: Deps, _env: Env) -> ContractResult<CampaignConfigResponse> {
    let config = CampaignConfig::load(deps.storage)?;

    Ok(CampaignConfigResponse {
        creation_fee_token: config.creation_fee_token.to_string(),
        creation_fee_amount: config.creation_fee_amount,
        creation_fee_recipient: config.creation_fee_recipient.to_string(),
        code_id: config.code_id,
        distribution_denom_whitelist: config.distribution_denom_whitelist.iter()
            .map(|d| Denom::from_cw20(d.clone()))
            .collect(),
        withdraw_fee_rate: config.withdraw_fee_rate,
        withdraw_fee_recipient: config.withdraw_fee_recipient.to_string(),
        deactivate_period: config.deactivate_period,
    })
}

pub fn get_booster_config(deps: Deps, _env: Env) -> ContractResult<BoosterConfigResponse> {
    let config = BoosterConfig::load(deps.storage)?;

    Ok(BoosterConfigResponse {
        booster_token: config.booster_token.to_string(),
        drop_booster_ratio: config.drop_ratio,
        activity_booster_ratio: config.activity_ratio,
        plus_booster_ratio: config.plus_ratio,
        activity_booster_multiplier: config.activity_multiplier,
        min_participation_count: config.min_participation_count,
    })
}

pub fn get_campaign(deps: Deps, _env: Env, address: String) -> ContractResult<CampaignResponse> {
    let campaign = Campaign::load(
        deps.storage,
        &deps.api.addr_validate(address.as_str())?,
    )?;

    Ok(CampaignResponse {
        code_id: campaign.code_id,
        address: campaign.address.to_string(),
        creator: campaign.creator.to_string(),
        created_height: campaign.created_height,
    })
}

pub fn query_campaign(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<CampaignsResponse> {
    let campaigns = Campaign::query(
        deps.storage,
        start_after,
        limit,
        order_by,
    )?;

    Ok(campaigns)
}