use cosmwasm_std::{Deps, Env};

use valkyrie::campaign_manager::query_msgs::{CampaignResponse, CampaignsResponse, ConfigResponse};
use valkyrie::common::{ContractResult, Denom, OrderBy};

use crate::states::*;

pub fn get_config(deps: Deps, _env: Env) -> ContractResult<ConfigResponse> {
    let config = Config::load(deps.storage)?;

    Ok(ConfigResponse {
        governance: config.governance.to_string(),
        fund_manager: config.fund_manager.to_string(),
        terraswap_router: config.terraswap_router.to_string(),
        creation_fee_token: config.creation_fee_token.to_string(),
        creation_fee_amount: config.creation_fee_amount,
        creation_fee_recipient: config.creation_fee_recipient.to_string(),
        code_id: config.code_id,
        withdraw_fee_rate: config.withdraw_fee_rate,
        withdraw_fee_recipient: config.withdraw_fee_recipient.to_string(),
        deactivate_period: config.deactivate_period,
        key_denom: Denom::from_cw20(config.key_denom),
        referral_reward_token: config.referral_reward_token.to_string(),
        min_referral_reward_deposit_rate: config.min_referral_reward_deposit_rate,
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