use crate::campaign::enumerations::Referrer;
use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::common::{Denom, ExecutionMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfigMsg {
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub distribution_denom: Denom,
    pub distribution_amounts: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateContractConfig {
        admin: Option<String>,
        proxies: Option<Vec<String>>,
    },
    UpdateCampaignInfo {
        title: Option<String>,
        description: Option<String>,
        url: Option<String>,
        parameter_key: Option<String>,
        executions: Option<Vec<ExecutionMsg>>,
    },
    UpdateDistributionConfig {
        denom: Denom,
        amounts: Vec<Uint128>,
    },
    UpdateActivation {
        active: bool,
    },
    EnableBooster {
        drop_booster_amount: Uint128,
        activity_booster_amount: Uint128,
        plus_booster_amount: Uint128,
        activity_booster_multiplier: Decimal,
    },
    DisableBooster {},
    Withdraw {
        denom: Denom,
        amount: Option<Uint128>,
    },
    ClaimParticipationReward {},
    ClaimBoosterReward {},
    Participate {
        actor: String,
        referrer: Option<Referrer>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributeResult {
    pub distributions: Vec<Distribution>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Distribution {
    pub address: String,
    pub distance: u64,
    pub reward_denom: Denom,
    pub reward_amount: Uint128,
    pub activity_boost_amount: Uint128,
    pub plus_boost_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
