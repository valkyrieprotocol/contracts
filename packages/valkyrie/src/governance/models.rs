use cosmwasm_std::{Addr, Binary, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enumerations::{PollStatus, VoteOption};

// Models

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteInfo {
    pub voter: Addr,
    pub option: VoteOption,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionMsg {
    pub order: u64,
    pub contract: Addr,
    pub msg: Binary,
}


// Responses

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ContractConfigResponse {
    pub token_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StakingStateResponse {
    pub total_share: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerStateResponse {
    pub balance: Uint128,
    pub share: Uint128,
    pub votes: Vec<(u64, VoteInfo)>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PollConfigResponse {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub execution_delay_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PollStateResponse {
    pub poll_count: u64,
    pub total_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollResponse {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub executions: Option<Vec<ExecutionMsg>>,
    pub creator: Addr,
    pub deposit_amount: Uint128,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub status: PollStatus,
    pub staked_amount: Option<Uint128>,
    pub total_balance_at_end_poll: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollsResponse {
    pub polls: Vec<PollResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollCountResponse {
    pub poll_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotersResponseItem {
    pub voter: Addr,
    pub vote: VoteOption,
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotersResponse {
    pub voters: Vec<VotersResponseItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValkyrieConfigResponse {
    pub campaign_code_whitelist: Vec<u64>,
    pub boost_contract: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignCodeInfoResponse {
    pub code_id: u64,
    pub source_code_url: String,
    pub description: String,
    pub maintainer: Option<String>,
}