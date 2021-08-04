use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128, Binary};
use crate::common::{Denom, ExecutionMsg};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
    pub fund_manager: String,
    pub terraswap_router: String,
    pub creation_fee_token: String,
    pub creation_fee_amount: Uint128,
    pub creation_fee_recipient: String,
    pub code_id: u64,
    pub withdraw_fee_rate: Decimal,
    pub withdraw_fee_recipient: String,
    pub deactivate_period: u64,
    pub key_denom: Denom,
    pub referral_reward_token: String,
    pub min_referral_reward_deposit_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        governance: Option<String>,
        fund_manager: Option<String>,
        terraswap_router: Option<String>,
        creation_fee_token: Option<String>,
        creation_fee_amount: Option<Uint128>,
        creation_fee_recipient: Option<String>,
        code_id: Option<u64>,
        withdraw_fee_rate: Option<Decimal>,
        withdraw_fee_recipient: Option<String>,
        deactivate_period: Option<u64>,
        key_denom: Option<Denom>,
        referral_reward_token: Option<String>,
        min_referral_reward_deposit_rate: Option<Decimal>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    CreateCampaign {
        config_msg: Binary,
        ticket_amount: u64,
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
    pub ticket_amount: u64,
    pub qualifier: Option<String>,
    pub executions: Vec<ExecutionMsg>,
    pub referral_reward_token: String,
}
