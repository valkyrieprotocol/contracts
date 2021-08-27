use cosmwasm_std::Uint128;
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie_qualifier::{QualificationMsg, QualifiedContinueOption};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub continue_option_on_fail: QualifiedContinueOption,
    pub min_token_balances: Vec<(Denom, Uint128)>,
    pub min_luna_staking: Uint128,
    pub participation_limit: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Qualify(QualificationMsg),
    UpdateConfig {
        admin: Option<String>,
        continue_option_on_fail: Option<QualifiedContinueOption>,
    },
    UpdateRequirement {
        min_token_balances: Option<Vec<(Denom, Uint128)>>,
        min_luna_staking: Option<Uint128>,
        participation_limit: Option<u64>, //zero is un-limit
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Qualify(QualificationMsg),
    Requirement {},
    Config {},
}
