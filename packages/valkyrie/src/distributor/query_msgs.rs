use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    Distributions {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfigResponse {
    pub admins: Vec<String>,
    pub managing_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub balance: Uint128,
    pub locked_amount: Uint128,
    pub distributed_amount: Uint128,
    pub free_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionsResponse {
    pub distributions: Vec<DistributionResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionResponse {
    pub id: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub recipient: String,
    pub amount: Uint128,
    pub released_amount: Uint128,
    pub distributable_amount: Uint128,
    pub distributed_amount: Uint128,
}
