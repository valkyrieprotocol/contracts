use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128, Binary};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub managing_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: Option<String>,
    },
    ApproveAdminNominee {},
    RegisterDistribution {
        start_height: u64,
        end_height: u64,
        recipient: String,
        amount: Uint128,
        message: Option<Binary>,
    },
    UpdateDistribution {
        id: u64,
        start_height: Option<u64>,
        end_height: Option<u64>,
        amount: Option<Uint128>,
        message: Option<Binary>,
    },
    RemoveDistributionMessage {
        id: u64,
    },
    Distribute {
        id: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
