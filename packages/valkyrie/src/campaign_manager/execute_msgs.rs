use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128, Binary};
use crate::common::{Denom, ExecutionMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
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
    pub referral_reward_limit_option: ReferralRewardLimitOptionMsg,
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
        fund_manager: Option<String>,
        terraswap_router: Option<String>,
        code_id: Option<u64>,
        deposit_fee_rate: Option<Decimal>,
        withdraw_fee_rate: Option<Decimal>,
        withdraw_fee_recipient: Option<String>,
        deactivate_period: Option<u64>,
        key_denom: Option<Denom>,
        referral_reward_token: Option<String>,
        min_referral_reward_deposit_rate: Option<Decimal>,
    },
    UpdateReferralRewardLimitOption {
        overflow_amount_recipient: Option<String>,
        base_count: Option<u8>,
        percent_for_governance_staking: Option<u16>,
    },
    SetReuseOverflowAmount {},
    CreateCampaign {
        config_msg: Binary,
        collateral_denom: Option<Denom>,
        collateral_amount: Option<Uint128>,
        collateral_lock_period: Option<u64>,
        qualifier: Option<String>,
        executions: Vec<ExecutionMsg>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInstantiateMsg {
    pub governance: String,
    pub campaign_manager: String,
    pub fund_manager: String,
    pub admin: String,
    pub creator: String,
    pub config_msg: Binary,
    pub collateral_denom: Option<Denom>,
    pub collateral_amount: Uint128,
    pub collateral_lock_period: u64,
    pub qualifier: Option<String>,
    pub executions: Vec<ExecutionMsg>,
    pub referral_reward_token: String,
}
