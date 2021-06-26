use cosmwasm_std::{Uint128, Binary, Decimal};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::campaign::enumerations::Denom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
    pub token_contract: String,
    pub distributor: String,
    pub campaign_code_id: u64,
    pub creation_fee_amount: Uint128,
    pub reward_withdraw_burn_rate: Decimal,
    pub campaign_deactivate_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateFactoryConfig {
        campaign_code_id: Option<u64>,
        creation_fee_amount: Option<Uint128>,
    },
    UpdateCampaignConfig {
        reward_withdraw_burn_rate: Option<Decimal>,
        campaign_deactivate_period: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    CreateCampaign {
        title: String,
        url: String,
        description: String,
        parameter_key: String,
        distribution_denom: Denom,
        distribution_amounts: Vec<Uint128>,
    },
}