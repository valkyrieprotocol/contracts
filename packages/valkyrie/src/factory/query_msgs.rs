use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    FactoryConfig {},
    CampaignConfig {},
    Campaign {
        address: String,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct FactoryConfigResponse {
    pub governance: String,
    pub token_contract: String,
    pub distributor: String,
    pub burn_contract: String,
    pub campaign_code_id: u64,
    pub creation_fee_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignResponse {
    pub code_id: u64,
    pub address: String,
    pub creator: String,
    pub created_block: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfigResponse {
    pub reward_withdraw_burn_rate: Decimal,
    pub campaign_deactivate_period: u64,
}