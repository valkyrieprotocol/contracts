use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enumerations::VoteOption;
use super::models::ExecutionMsg;
use crate::governance::enumerations::PollStatus;
use crate::common::OrderBy;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub contract_config: ContractConfigInitMsg,
    pub poll_config: PollConfigInitMsg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfigInitMsg {
    pub token_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollConfigInitMsg {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub execution_delay_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UnstakeVotingToken { amount: Option<Uint128> },
    UpdatePollConfig {
        quorum: Option<Decimal>,
        threshold: Option<Decimal>,
        voting_period: Option<u64>,
        execution_delay_period: Option<u64>,
        expiration_period: Option<u64>,
        proposal_deposit: Option<Uint128>,
        snapshot_period: Option<u64>,
    },
    CastVote {
        poll_id: u64,
        vote: VoteOption,
        amount: Uint128,
    },
    EndPoll { poll_id: u64 },
    ExecutePoll { poll_id: u64 },
    #[cfg(feature = "expire")]
    ExpirePoll { poll_id: u64 },
    SnapshotPoll { poll_id: u64 },
    UpdateValkyrieConfig {
        boost_contract: Option<Addr>,
    },
    AddCampaignCodeWhitelist {
        code_id: u64,
        source_code_url: String,
        description: String,
        maintainer: Option<String>,
    },
    RemoveCampaignCodeWhitelist {
        code_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    StakeVotingToken {},
    CreatePoll {
        title: String,
        description: String,
        link: Option<String>,
        execution: Option<Vec<ExecutionMsg>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ContractConfig {},
    PollConfig {},
    PollState {},
    Poll { poll_id: u64 },
    Polls {
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    Voters {
        poll_id: u64,
        start_after: Option<Addr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    StakingState {},
    StakerState { address: Addr },
    ValkyrieConfig {},
    CampaignCodeInfo { code_id: u64 },
}