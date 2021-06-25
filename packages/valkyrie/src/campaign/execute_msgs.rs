use crate::campaign::enumerations::{Denom, Referrer};
use cosmwasm_std::{Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
    pub distributor: String,
    pub token_contract: String,
    pub title: String,
    pub url: String,
    pub description: String,
    pub parameter_key: String,
    pub distribution_denom: Denom,
    pub distribution_amounts: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateCampaignInfo {
        title: Option<String>,
        url: Option<String>,
        description: Option<String>,
    },
    UpdateDistributionConfig {
        denom: Denom,
        amounts: Vec<Uint128>,
    },
    UpdateAdmin {
        address: String,
    },
    UpdateActivation {
        active: bool,
    },
    RegisterBooster {
        drop_booster_amount: Uint128,
        activity_booster_amount: Uint128,
        plus_booster_amount: Uint128,
    },
    DeregisterBooster {},
    WithdrawReward {
        denom: Denom,
        amount: Option<Uint128>,
    },
    ClaimReward {},
    Participate {
        referrer: Option<Referrer>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributeResult {
    pub actor_address: String,
    pub reward_denom: Denom,
    pub configured_reward_amount: Uint128,
    pub distributed_reward_amount: Uint128,
    pub distributions: Vec<Distribution>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Distribution {
    pub address: String,
    pub distance: Uint64,
    pub amount: Uint128,
}
