use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::common::OrderBy;

use super::enumerations::{PollStatus, VoteOption};
use super::models::ExecutionMsg;

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
    UpdateContractConfig {
        admin: Option<Addr>,
        boost_contract: Option<Addr>,
    },
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
    ExpirePoll { poll_id: u64 },
    SnapshotPoll { poll_id: u64 },
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