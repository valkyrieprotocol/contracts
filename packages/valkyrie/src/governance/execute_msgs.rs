use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enumerations::VoteOption;
use super::models::ExecutionMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub contract_config: ContractConfigInitMsg,
    pub poll_config: PollConfigInitMsg,
    pub valkyrie_config: ValkyrieConfigInitMsg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfigInitMsg {
    pub token_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollConfigInitMsg {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub execution_delay_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValkyrieConfigInitMsg {
    pub burn_contract: String,
    pub reward_withdraw_burn_rate: Decimal,
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
    SnapshotPoll { poll_id: u64 },
    UpdateValkyrieConfig {
        burn_contract: Option<String>,
        reward_withdraw_burn_rate: Option<Decimal>,
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