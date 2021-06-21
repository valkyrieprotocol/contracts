use crate::common::OrderBy;
use cosmwasm_std::{Uint64, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Campaign {
        address: String,
    }
}

pub struct ConfigResponse {
    pub governance: String,
    pub token_contract: String,
    pub campaign_code_id: Uint64,
    pub creation_fee_amount: Uint128,
}

pub struct CampaignResponse {
    pub code_id: Uint64,
    pub address: String,
    pub creator: String,
    pub created_block: Uint64,
}