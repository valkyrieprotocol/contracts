use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128, Binary};
use crate::common::Denom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
    pub terraswap_router: String,
    pub code_id: u64,
    pub add_pool_fee_rate: Decimal,
    pub add_pool_min_referral_reward_rate: Decimal,
    pub remove_pool_fee_rate: Decimal,
    pub fee_burn_ratio: Decimal,
    pub fee_recipient: String,
    pub deactivate_period: u64,
    pub key_denom: Denom,
    pub valkyrie_token: String,
    pub referral_reward_limit_option: ReferralRewardLimitOptionMsg,
    pub contract_admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReferralRewardLimitOptionMsg {
    pub overflow_amount_recipient: Option<String>,
    pub base_count: u8,
    pub percent_for_governance_staking: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        governance: Option<String>,
        valkyrie_token: Option<String>,
        terraswap_router: Option<String>,
        code_id: Option<u64>,
        add_pool_fee_rate: Option<Decimal>,
        add_pool_min_referral_reward_rate: Option<Decimal>,
        remove_pool_fee_rate: Option<Decimal>,
        fee_burn_ratio: Option<Decimal>,
        fee_recipient: Option<String>,
        deactivate_period: Option<u64>,
        key_denom: Option<Denom>,
        contract_admin: Option<String>,
    },
    ApproveContractAdminNominee {},
    UpdateReferralRewardLimitOption {
        overflow_amount_recipient: Option<String>,
        base_count: Option<u8>,
        percent_for_governance_staking: Option<u16>,
    },
    SetReuseOverflowAmount {},
    CreateCampaign {
        config_msg: Binary,
        deposit_denom: Option<Denom>,
        deposit_amount: Option<Uint128>,
        deposit_lock_period: Option<u64>,
        qualifier: Option<String>,
        qualification_description: Option<String>,
    },
    SpendFee {
        amount: Option<Uint128>,
    },
    SwapFee {
        denom: Denom,
        amount: Option<Uint128>,
        route: Option<Vec<Denom>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub contract_admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInstantiateMsg {
    pub governance: String,
    pub campaign_manager: String,
    pub admin: String,
    pub creator: String,
    pub config_msg: Binary,
    pub deposit_denom: Option<Denom>,
    pub deposit_amount: Uint128,
    pub deposit_lock_period: u64,
    pub qualifier: Option<String>,
    pub qualification_description: Option<String>,
    pub referral_reward_token: String,
}
