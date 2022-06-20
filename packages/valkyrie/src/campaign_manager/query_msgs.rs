use cosmwasm_std::Decimal;
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance: String,
    pub valkyrie_token: String,
    pub valkyrie_proxy: String,
    pub code_id: u64,
    pub add_pool_fee_rate: Decimal,
    pub add_pool_min_referral_reward_rate: Decimal,
    pub remove_pool_fee_rate: Decimal,
    pub fee_burn_ratio: Decimal,
    pub fee_recipient: String,
    pub deactivate_period: u64,
    pub key_denom: Denom,
    pub contract_admin: String,
    pub vp_token: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for ConfigResponse {
    fn default() -> Self {
        ConfigResponse {
            governance: governance::GOVERNANCE.to_string(),
            valkyrie_token: VALKYRIE_TOKEN.to_string(),
            valkyrie_proxy: VALKYRIE_PROXY.to_string(),
            code_id: CAMPAIGN_CODE_ID,
            add_pool_fee_rate: Decimal::percent(ADD_POOL_FEE_RATE_PERCENT),
            add_pool_min_referral_reward_rate: Decimal::percent(ADD_POOL_MIN_REFERRAL_REWARD_RATE_PERCENT),
            remove_pool_fee_rate: Decimal::percent(REMOVE_POOL_FEE_RATE_PERCENT),
            fee_burn_ratio: Decimal::percent(FEE_BURN_RATIO_PERCENT),
            fee_recipient: FEE_RECIPIENT.to_string(),
            deactivate_period: CAMPAIGN_DEACTIVATE_PERIOD,
            key_denom: Denom::Native(KEY_DENOM_NATIVE.to_string()),
            contract_admin: governance::GOVERNANCE.to_string(),
            vp_token: VALKYRIE_TICKET_TOKEN.to_string(),
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
