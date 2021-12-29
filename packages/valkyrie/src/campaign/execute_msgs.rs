use crate::campaign::enumerations::Referrer;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::common::Denom;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfigMsg {
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub participation_reward_denom: Denom,
    pub participation_reward_amount: Uint128,
    pub participation_reward_lock_period: u64,
    pub referral_reward_amounts: Vec<Uint128>,
    pub referral_reward_lock_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateCampaignConfig {
        title: Option<String>,
        description: Option<String>,
        url: Option<String>,
        parameter_key: Option<String>,
        deposit_amount: Option<Uint128>,
        deposit_lock_period: Option<u64>,
        qualifier: Option<String>,
        qualification_description: Option<String>,
        admin: Option<String>,
    },
    ApproveAdminNominee {
        address: String,
    },
    UpdateRewardConfig {
        participation_reward_amount: Option<Uint128>,
        participation_reward_lock_period: Option<u64>,
        referral_reward_amounts: Option<Vec<Uint128>>,
        referral_reward_lock_period: Option<u64>,
    },
    UpdateActivation {
        active: bool,
    },
    SetNoQualification {},
    AddRewardPool {
        participation_reward_amount: Uint128,
        referral_reward_amount: Uint128,
    },
    RemoveRewardPool {
        denom: Denom,
        amount: Option<Uint128>,
    },
    ClaimParticipationReward {},
    ClaimReferralReward {},
    Participate {
        actor: String,
        referrer: Option<Referrer>,
    },
    Deposit {},
    Withdraw {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributeResult {
    pub participation_reward_denom: Denom,
    pub participation_reward_amount: Uint128,
    pub referral_rewards: Vec<ReferralReward>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReferralReward {
    pub address: String,
    pub distance: u64,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
