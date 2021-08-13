use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::common::{OrderBy, Denom};

#[cfg(not(target_arch = "wasm32"))]
use crate::test_constants::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::test_constants::campaign_manager::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    ReferralRewardLimitOption {},
    Campaign {
        address: String,
    },
    Campaigns {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    ReferralRewardLimitAmount {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance: String,
    pub fund_manager: String,
    pub terraswap_router: String,
    pub code_id: u64,
    pub deposit_fee_rate: Decimal,
    pub withdraw_fee_rate: Decimal,
    pub withdraw_fee_recipient: String,
    pub deactivate_period: u64,
    pub key_denom: Denom,
    pub referral_reward_token: String,
    pub min_referral_reward_deposit_rate: Decimal,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for ConfigResponse {
    fn default() -> Self {
        ConfigResponse {
            governance: governance::GOVERNANCE.to_string(),
            fund_manager: fund_manager::FUND_MANAGER.to_string(),
            terraswap_router: TERRASWAP_ROUTER.to_string(),
            code_id: CAMPAIGN_CODE_ID,
            deposit_fee_rate: Decimal::percent(DEPOSIT_FEE_RATE_PERCENT),
            withdraw_fee_rate: Decimal::percent(WITHDRAW_FEE_RATE_PERCENT),
            withdraw_fee_recipient: fund_manager::FUND_MANAGER.to_string(),
            deactivate_period: CAMPAIGN_DEACTIVATE_PERIOD,
            key_denom: Denom::Native(KEY_DENOM_NATIVE.to_string()),
            referral_reward_token: REFERRAL_REWARD_TOKEN.to_string(),
            min_referral_reward_deposit_rate: Decimal::percent(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReferralRewardLimitOptionResponse {
    pub overflow_amount_recipient: Option<String>,
    pub base_count: u8,
    pub percent_for_governance_staking: u16,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for ReferralRewardLimitOptionResponse {
    fn default() -> Self {
        ReferralRewardLimitOptionResponse {
            overflow_amount_recipient: None,
            base_count: REFERRAL_REWARD_LIMIT_BASE_COUNT,
            percent_for_governance_staking: REFERRAL_REWARD_LIMIT_STAKING_PERCENT,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignResponse {
    pub code_id: u64,
    pub address: String,
    pub creator: String,
    pub created_height: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignsResponse {
    pub campaigns: Vec<CampaignResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ReferralRewardLimitAmountResponse {
    pub address: String,
    pub amount: Uint128,
}
