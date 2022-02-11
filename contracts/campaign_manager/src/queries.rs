use cosmwasm_std::{Deps, Env};

use valkyrie::campaign_manager::query_msgs::{CampaignResponse, CampaignsResponse, ConfigResponse, ReferralRewardLimitOptionResponse};
use valkyrie::common::{ContractResult, Denom, OrderBy};

use crate::states::*;

pub fn get_config(deps: Deps, _env: Env) -> ContractResult<ConfigResponse> {
    let config = Config::load(deps.storage)?;

    Ok(ConfigResponse {
        governance: config.governance.to_string(),
        valkyrie_token: config.valkyrie_token.to_string(),
        terraswap_router: config.terraswap_router.to_string(),
        code_id: config.code_id,
        add_pool_fee_rate: config.add_pool_fee_rate,
        add_pool_min_referral_reward_rate: config.add_pool_min_referral_reward_rate,
        remove_pool_fee_rate: config.remove_pool_fee_rate,
        fee_burn_ratio: config.fee_burn_ratio,
        fee_recipient: config.fee_recipient.to_string(),
        deactivate_period: config.deactivate_period,
        key_denom: Denom::from_cw20(config.key_denom),
        contract_admin: config.contract_admin.to_string(),
        vp_token: config.vp_token.to_string(),
    })
}

pub fn get_referral_reward_limit_option(
    deps: Deps,
    _env: Env,
) -> ContractResult<ReferralRewardLimitOptionResponse> {
    let option = ReferralRewardLimitOption::load(deps.storage)?;

    Ok(ReferralRewardLimitOptionResponse {
        overflow_amount_recipient: option.overflow_amount_recipient.map(|r| r.to_string()),
        base_count: option.base_count,
        percent_for_governance_staking: option.percent_for_governance_staking,
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
