use cosmwasm_std::{Uint128, Uint64, Decimal};
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
    pub campaign_code_id: Uint64,
    pub creation_fee_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignResponse {
    pub code_id: Uint64,
    pub address: String,
    pub creator: String,
    pub created_block: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfigResponse {
    pub reward_withdraw_burn_rate: Decimal,
    pub campaign_deactivate_period: Uint64,
}