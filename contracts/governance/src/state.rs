use std::cmp::Ordering;

use cosmwasm_std::{Binary, CanonicalAddr, Decimal, Storage, Uint128, StdResult};
use cosmwasm_storage::{
    bucket, Bucket, bucket_read, ReadonlyBucket, ReadonlySingleton, singleton, Singleton,
    singleton_read,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::governance::enumerations::{PollStatus, OrderBy};
use valkyrie::governance::models::VoterInfo;


static PREFIX_BANK: &[u8] = b"bank";


static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub valkyrie_token: CanonicalAddr,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

impl Config {
    pub fn singleton<S: Storage>(storage: &mut S) -> Singleton<Config> {
        singleton(storage, KEY_CONFIG)
    }

    pub fn singleton_read<S: Storage>(storage: &S) -> ReadonlySingleton<Config> {
        singleton_read(storage, KEY_CONFIG)
    }
}


static KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub contract_addr: CanonicalAddr,
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

impl State {
    pub fn singleton<S: Storage>(storage: &mut S) -> Singleton<State> {
        singleton(storage, KEY_STATE)
    }

    pub fn singleton_read<S: Storage>(storage: &S) -> ReadonlySingleton<State> {
        singleton_read(storage, KEY_STATE)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128,
    // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}


static PREFIX_POLL: &[u8] = b"poll";
static PREFIX_POLL_INDEXER: &[u8] = b"poll_indexer";
static PREFIX_POLL_VOTER: &[u8] = b"poll_voter";

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub executions: Option<Vec<Execution>>,
    pub creator: CanonicalAddr,
    pub deposit_amount: Uint128,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub status: PollStatus,
    pub staked_amount: Option<Uint128>,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
}

impl Poll {
    pub fn bucket<S: Storage>(storage: &mut S) -> Bucket<Poll> {
        bucket(storage, PREFIX_POLL)
    }

    pub fn bucket_read<S: Storage>(storage: &S) -> ReadonlyBucket<Poll> {
        bucket_read(storage, PREFIX_POLL)
    }

    pub fn indexer_bucket<'a, S: Storage>(
        storage: &'a mut S,
        status: &PollStatus,
    ) -> Bucket<'a, bool> {
        Bucket::multilevel(
            storage,
            &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
        )
    }

    pub fn voter_bucket<S: Storage>(storage: &mut S, poll_id: u64) -> Bucket<VoterInfo> {
        Bucket::multilevel(
            storage,
            &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()],
        )
    }

    pub fn voter_bucket_read<S: ReadonlyStorage>(
        storage: &S,
        poll_id: u64,
    ) -> ReadonlyBucket<VoterInfo> {
        ReadonlyBucket::multilevel(
            storage,
            &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()],
        )
    }

    pub fn read<'a, S: ReadonlyStorage>(
        storage: &'a S,
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Poll>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };

        if let Some(status) = filter {
            let poll_indexer: ReadonlyBucket<'a, bool> = ReadonlyBucket::multilevel(
                storage,
                &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
            );
            poll_indexer
                .range(start.as_deref(), end.as_deref(), order_by.into())
                .take(limit)
                .map(|item| {
                    let (k, _) = item?;
                    poll_read(storage).load(&k)
                })
                .collect()
        } else {
            let polls: ReadonlyBucket<'a, Poll> = ReadonlyBucket::new(
                storage,
                PREFIX_POLL,
            );

            polls
                .range(start.as_deref(), end.as_deref(), order_by.into())
                .take(limit)
                .map(|item| {
                    let (_, v) = item?;
                    Ok(v)
                })
                .collect()
        }
    }

    pub fn read_voters<'a, S: ReadonlyStorage>(
        storage: &'a S,
        poll_id: u64,
        start_after: Option<CanonicalAddr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<CanonicalAddr, VoterInfo>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
        };

        let voters: ReadonlyBucket<'a, VoterInfo> = ReadonlyBucket::multilevel(
            storage,
            &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()],
        );

        voters
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| {
                let (k, v) = item?;
                Ok((CanonicalAddr::from(k), v))
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Execution {
    pub order: u64,
    pub contract: CanonicalAddr,
    pub msg: Binary,
}

impl PartialEq for Execution {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Eq for Execution {}

impl PartialOrd for Execution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Execution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| id.to_be_bytes().to_vec())
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start_addr(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_slice().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end_addr(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|addr| addr.as_slice().to_vec())
}