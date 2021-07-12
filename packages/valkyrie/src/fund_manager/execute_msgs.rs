use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;
use crate::common::Denom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub managing_token: String,
    pub terraswap_router: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        admins: Option<Vec<String>>,
        terraswap_router: Option<String>,
    },
    IncreaseAllowance {
        address: String,
        amount: Uint128,
    },
    DecreaseAllowance {
        address: String,
        amount: Option<Uint128>,
    },
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    Swap {
        denom: Denom,
        amount: Option<Uint128>,
        route: Option<Vec<Denom>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
