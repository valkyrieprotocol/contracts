use cosmwasm_std::{Deps, Env, Uint128, StdError};

use valkyrie::campaign_manager::query_msgs::{CampaignResponse, CampaignsResponse, ConfigResponse, ReferralRewardLimitAmountResponse, ReferralRewardLimitOptionResponse};
use valkyrie::common::{ContractResult, Denom, OrderBy};

use crate::states::*;
use valkyrie::governance::query_msgs::{QueryMsg, StakerStateResponse};

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
        deposit_fee_rate: config.deposit_fee_rate,
        withdraw_fee_rate: config.withdraw_fee_rate,
        withdraw_fee_recipient: config.withdraw_fee_recipient.to_string(),
        deactivate_period: config.deactivate_period,
        key_denom: Denom::from_cw20(config.key_denom),
        referral_reward_token: config.referral_reward_token.to_string(),
        min_referral_reward_deposit_rate: config.min_referral_reward_deposit_rate,
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

pub fn get_referral_reward_limit_amount(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<ReferralRewardLimitAmountResponse> {
    let address = deps.api.addr_validate(address.as_str())?;

    let config = Config::load(deps.storage)?;
    let option = ReferralRewardLimitOption::load(deps.storage)?;

    let gov_staker_state: StakerStateResponse = deps.querier.query_wasm_smart(
        config.governance,
        &QueryMsg::StakerState {
            address: address.to_string(),
        },
    )?;
    let gov_staking_amount = gov_staker_state.balance;

    let amount = gov_staking_amount
        .checked_mul(Uint128::from(100 * option.percent_for_governance_staking))?
        .checked_div(Uint128::new(100))
        .map_err(|e| StdError::divide_by_zero(e))?;

    Ok(ReferralRewardLimitAmountResponse {
        address: address.to_string(),
        amount,
    })
}
