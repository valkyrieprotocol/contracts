use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::common::{OrderBy, Denom};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ContractConfig {},
    CampaignConfig {},
    BoosterConfig {},
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
pub struct ContractConfigResponse {
    pub governance: String,
    pub fund_manager: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default, JsonSchema)]
pub struct CampaignConfigResponse {
    pub creation_fee_token: String,
    pub creation_fee_amount: Uint128,
    pub creation_fee_recipient: String,
    pub code_id: u64,
    pub distribution_denom_whitelist: Vec<Denom>,
    pub withdraw_fee_rate: Decimal,
    pub withdraw_fee_recipient: String,
    pub deactivate_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct BoosterConfigResponse {
    pub booster_token: String,
    pub drop_booster_ratio: Decimal,
    pub activity_booster_ratio: Decimal,
    pub plus_booster_ratio: Decimal,
    pub activity_booster_multiplier: Decimal,
    pub min_participation_count: u64,
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
