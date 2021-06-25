use crate::common::OrderBy;

use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ContractConfig {},
    DistributorInfo {
        distributor: String,
    },
    DistributorInfos {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfigResponse {
    pub governance: String,
    pub token_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributorInfoResponse {
    pub distributor: String,
    pub spend_limit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributorInfosResponse {
    pub distributors: Vec<DistributorInfoResponse>,
}
