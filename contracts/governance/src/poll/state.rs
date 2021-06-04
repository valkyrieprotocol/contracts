use std::cmp::Ordering;

use cosmwasm_std::{Binary, CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, bucket_read, ReadonlyBucket, ReadonlySingleton, Singleton, singleton_read};

use valkyrie::common::OrderBy;
use valkyrie::governance::enumerations::PollStatus;
use valkyrie::governance::models::VoterInfo;
use crate::staking::state::StakingState;

static KEY_POLL_CONFIG: &[u8] = b"poll-config";
static KEY_POLL_STATE: &[u8] = b"poll-state";
static PREFIX_POLL: &[u8] = b"poll";
static PREFIX_POLL_INDEXER: &[u8] = b"poll-indexer";
static PREFIX_POLL_VOTER: &[u8] = b"poll-voter";

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollConfig {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub execution_delay_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

impl PollConfig {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<PollConfig> {
        singleton(storage, KEY_CONFIG)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<PollConfig> {
        singleton_read(storage, KEY_CONFIG)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        PollConfig::singleton(storage).save(self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<PollConfig> {
        PollConfig::singleton_read(storage).load()
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollState {
    pub poll_count: u64,
    pub total_deposit: Uint128,
}

impl PollState {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<PollState> {
        singleton(storage, KEY_POLL_STATE)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<PollState> {
        singleton_read(storage, KEY_POLL_STATE)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        PollState::singleton(storage).save(self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<PollState> {
        PollState::singleton_read(storage).load()
    }
}

pub fn get_poll_id(storage: &mut dyn Storage, deposit_amount: &Uint128) -> StdResult<u64> {
    let mut poll_state = PollState::load(storage)?;
    let poll_id = poll_state.poll_count + 1;

    poll_state.poll_count += 1;
    poll_state.total_deposit += deposit_amount;
    poll_state.save(storage)?;

    Ok(poll_id)
}


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
    pub fn bucket(storage: &mut dyn Storage) -> Bucket<Poll> {
        bucket(storage, PREFIX_POLL)
    }

    pub fn bucket_read(storage: &dyn Storage) -> ReadonlyBucket<Poll> {
        bucket_read(storage, PREFIX_POLL)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        Poll::bucket(storage).save(&self.id.to_be_bytes(), &self)
    }

    pub fn load(storage: &dyn Storage, poll_id: &u64) -> StdResult<Poll> {
        Poll::bucket_read(storage).load(&poll_id.to_be_bytes())
    }

    pub fn indexer_bucket<'a>(
        storage: &'a mut dyn Storage,
        status: &PollStatus,
    ) -> Bucket<'a, bool> {
        Bucket::multilevel(
            storage,
            &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
        )
    }

    pub fn voter_bucket(storage: &mut dyn Storage, poll_id: &u64) -> Bucket<VoterInfo> {
        Bucket::multilevel(
            storage,
            &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()],
        )
    }

    pub fn remove_voter(storage: &mut dyn Storage, poll_id: &u64, voter: &CanonicalAddr) {
        Poll::voter_bucket(storage, poll_id).remove(voter.as_slice())
    }

    pub fn voter_bucket_read(
        storage: &dyn Storage,
        poll_id: u64,
    ) -> ReadonlyBucket<VoterInfo> {
        ReadonlyBucket::multilevel(
            storage,
            &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()],
        )
    }

    pub fn read<'a>(
        storage: &'a dyn Storage,
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
                    Poll::bucket_read(storage).load(&k)
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

    pub fn read_voters<'a>(
        storage: &'a dyn Storage,
        poll_id: u64,
        start_after: Option<CanonicalAddr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(CanonicalAddr, VoterInfo)>> {
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

    pub fn in_progress(&self, block_height: u64) -> bool {
        poll.status == PollStatus::InProgress && block_height <= poll.end_height
    }

    pub fn load_voter(&self, storage: &dyn Storage, address: &CanonicalAddr) -> StdResult<VoterInfo> {
        Poll::voter_bucket_read(storage, self.id).load(address.as_slice())
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