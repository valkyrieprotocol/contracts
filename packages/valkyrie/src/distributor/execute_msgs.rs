use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
    pub token_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Spend {
        recipient: String,
        amount: Uint128,
    },
    AddDistributor {
        distributor: String,
        spend_limit: Uint128,
    },
    RemoveDistributor {
        distributor: String,
    },
}
