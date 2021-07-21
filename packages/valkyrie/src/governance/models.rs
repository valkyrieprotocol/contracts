use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enumerations::VoteOption;

// Models

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteInfoMsg {
    pub voter: String,
    pub option: VoteOption,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionMsg {
    pub order: u64,
    pub contract: String,
    pub msg: Binary,
}
