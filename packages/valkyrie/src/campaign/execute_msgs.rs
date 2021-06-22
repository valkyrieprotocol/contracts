use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::campaign::enumerations::{Denom, Referrer};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance: String,
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
pub enum Cw20HookMsg {
}