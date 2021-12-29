use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token: String,
    pub pair: String,
    pub lp_token: String,
    pub whitelisted_contracts: Vec<String>,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Unbond {
        amount: Uint128,
    },
    /// Withdraw pending rewards
    Withdraw {},
    AutoStake {
        token_amount: Uint128,
        slippage_tolerance: Option<Decimal>,
    },
    AutoStakeHook {
        staker_addr: String,
        already_staked_amount: Uint128,
    },
    UpdateConfig {
        token: Option<String>,
        pair: Option<String>,
        lp_token: Option<String>,
        admin: Option<String>,
        whitelisted_contracts: Option<Vec<String>>,
        distribution_schedule: Option<Vec<(u64, u64, Uint128)>>,
    },
    MigrateReward {
        recipient: String,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Bond {},
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub admin: String,
    pub whitelisted_contracts: Vec<String>,
}