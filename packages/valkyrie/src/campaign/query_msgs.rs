use cosmwasm_std::{Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::campaign::enumerations::Referrer;
use crate::common::{Denom, OrderBy};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CampaignConfig {},
    RewardConfig {},
    CampaignState {},
    ShareUrl {
        address: String,
    },
    GetAddressFromReferrer {
        referrer: Referrer,
    },
    ReferralRewardLimitAmount {
        address: String,
    },
    Actor {
        address: String,
    },
    Actors {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    Deposit {
        address: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignConfigResponse {
    pub governance: String,
    pub campaign_manager: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub deposit_denom: Option<Denom>,
    pub deposit_amount: Uint128,
    pub deposit_lock_period: u64,
    pub qualifier: Option<String>,
    pub qualification_description: Option<String>,
    pub admin: String,
    pub creator: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct RewardConfigResponse {
    pub participation_reward_denom: Denom,
    pub participation_reward_amount: Uint128,
    pub referral_reward_token: String,
    pub referral_reward_amounts: Vec<Uint128>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CampaignStateResponse {
    pub actor_count: u64,
    pub participation_count: u64,
    pub cumulative_participation_reward_amount: Uint128,
    pub cumulative_referral_reward_amount: Uint128,
    pub locked_balances: Vec<(Denom, Uint128)>,
    pub balances: Vec<(Denom, Uint128)>,
    pub deposit_amount: Uint128,
    pub is_active: bool,
    pub is_pending: bool,
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
pub struct ReferralRewardLimitAmount {
    pub address: String,
    pub limit_amount: Uint128,
    pub base_limit_amount: Uint128,
    pub actor_limit_amount: Uint128,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ActorResponse {
    pub address: String,
    pub referrer_address: Option<String>,
    pub participation_reward_amount: Uint128,
    pub referral_reward_amount: Uint128,
    pub cumulative_participation_reward_amount: Uint128,
    pub cumulative_referral_reward_amount: Uint128,
    pub participation_count: u64,
    pub referral_count: u64,
    pub last_participated_at: Timestamp,
}

impl ActorResponse {
    pub fn new(address: String, referrer: Option<String>) -> ActorResponse {
        ActorResponse {
            address,
            referrer_address: referrer,
            participation_reward_amount: Uint128::zero(),
            referral_reward_amount: Uint128::zero(),
            cumulative_participation_reward_amount: Uint128::zero(),
            cumulative_referral_reward_amount: Uint128::zero(),
            participation_count: 0,
            referral_count: 0,
            last_participated_at: Timestamp::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ActorsResponse {
    pub actors: Vec<ActorResponse>,
}