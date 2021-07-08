use cosmwasm_std::{Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::campaign::enumerations::Referrer;
use crate::common::{OrderBy, Denom, ExecutionMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ContractConfig {},
    CampaignInfo {},
    DistributionConfig {},
    CampaignState {},
    ActiveBooster {},
    PrevBooster {
        booster_id: u64,
    },
    PrevBoosters {
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    ShareUrl {
        address: String,
    },
    GetAddressFromReferrer {
        referrer: Referrer,
    },
    Participation {
        address: String,
    },
    Participations {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ContractConfigResponse {
    pub admin: String,
    pub governance: String,
    pub campaign_manager: String,
    pub fund_manager: String,
    pub proxies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignInfoResponse {
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub executions: Vec<ExecutionMsg>,
    pub creator: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct DistributionConfigResponse {
    pub denom: Denom,
    pub amounts: Vec<Uint128>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignStateResponse {
    pub participation_count: u64,
    pub cumulative_distribution_amount: Uint128,
    pub locked_balance: Uint128,
    pub balance: Uint128,
    pub is_active: bool,
    pub is_pending: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ActiveBoosterResponse {
    pub active_booster: Option<BoosterResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PrevBoostersResponse {
    pub prev_boosters: Vec<BoosterResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BoosterResponse {
    pub drop_booster: DropBoosterResponse,
    pub activity_booster: ActivityBoosterResponse,
    pub plus_booster: PlusBoosterResponse,
    pub boosted_at: Timestamp,
    pub finished_at: Option<Timestamp>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct DropBoosterResponse {
    pub assigned_amount: Uint128,
    pub calculated_amount: Uint128,
    pub spent_amount: Uint128,
    pub reward_amounts: Vec<Uint128>,
    pub snapped_participation_count: u64,
    pub snapped_distance_counts: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ActivityBoosterResponse {
    pub assigned_amount: Uint128,
    pub distributed_amount: Uint128,
    pub reward_amounts: Vec<Uint128>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PlusBoosterResponse {
    pub assigned_amount: Uint128,
    pub distributed_amount: Uint128,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ShareUrlResponse {
    pub address: String,
    pub compressed: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetAddressFromReferrerResponse {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ParticipationResponse {
    pub actor_address: String,
    pub referrer_address: Option<String>,
    pub reward_amount: Uint128,
    pub participated_at: Timestamp,
    pub drop_booster_amount: Uint128,
    pub activity_booster_amount: Uint128,
    pub plus_booster_amount: Uint128,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ParticipationsResponse {
    pub participations: Vec<ParticipationResponse>,
}