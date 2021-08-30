use std::fmt;

use cosmwasm_std::{Addr, Decimal, Deps, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::common::{OrderBy, Execution, ExecutionMsg};
use valkyrie::governance::enumerations::{PollStatus, VoteOption};
use valkyrie::governance::query_msgs::PollResponse;

use crate::common::states::load_available_balance;
use crate::staking::states::{StakerState, StakingState};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;


const POLL_CONFIG: Item<PollConfig> = Item::new("poll-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollConfig {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub execution_delay_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

impl PollConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        POLL_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<PollConfig> {
        POLL_CONFIG.load(storage)
    }
}


const POLL_STATE: Item<PollState> = Item::new("poll-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollState {
    pub poll_count: u64,
    pub total_deposit: Uint128,
}

impl PollState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        POLL_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<PollState> {
        POLL_STATE.load(storage)
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


const POLLS: Map<&[u8], Poll> = Map::new("poll");
const POLL_STATUS_INDEX: Map<(&[u8], &[u8]), bool> = Map::new("poll-status-index");
const POLL_VOTERS: Map<(&[u8], &[u8]), VoteInfo> = Map::new("poll-voter");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub executions: Vec<Execution>,
    pub creator: Addr,
    pub deposit_amount: Uint128,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub abstain_votes: Uint128,
    pub end_height: u64,
    pub status: PollStatus,
    pub snapped_staked_amount: Option<Uint128>,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,

    pub _status: Option<PollStatus>,
}

impl Poll {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        POLLS.save(storage, &self.id.to_be_bytes(), self)
    }

    pub fn save_with_index(&mut self, storage: &mut dyn Storage) -> StdResult<()> {
        let id = self.id.to_be_bytes();

        if self._status.is_some() {
            let prev_status = self._status.clone().unwrap();

            POLL_STATUS_INDEX.remove(storage, (&prev_status.to_string().as_bytes(), &id));
        }

        POLL_STATUS_INDEX.save(storage, (&self.status.to_string().as_bytes(), &id), &true)?;

        self._status = Some(self.status.clone());
        self.save(storage)
    }

    pub fn load(storage: &dyn Storage, poll_id: &u64) -> StdResult<Poll> {
        POLLS.load(storage, &poll_id.to_be_bytes())
    }

    pub fn may_load(storage: &dyn Storage, poll_id: &u64) -> StdResult<Option<Poll>> {
        POLLS.may_load(storage, &poll_id.to_be_bytes())
    }

    pub fn query<'a>(
        storage: &'a dyn Storage,
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Poll>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(|v| Bound::exclusive(v.to_be_bytes()));
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
            _ => (None, start_after, OrderBy::Desc),
        };

        if let Some(status) = filter {
            POLL_STATUS_INDEX.prefix(&status.to_string().as_bytes())
                .range(storage, min, max, order_by.into())
                .take(limit)
                .map(|item| {
                    let (k, _) = item?;
                    POLLS.load(storage, k.as_slice())
                })
                .collect()
        } else {
            POLLS.range(storage, min, max, order_by.into())
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
        poll_id: &u64,
        start_after: Option<Addr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(Addr, VoteInfo)>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(|v| Bound::exclusive(v.as_str().as_bytes()));
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
            _ => (None, start_after, OrderBy::Desc),
        };

        POLL_VOTERS
            .prefix(&poll_id.to_be_bytes())
            .range(storage, min, max, order_by.into())
            .take(limit)
            .map(|item| {
                let (k, v) = item?;
                Ok((Addr::unchecked(std::str::from_utf8(&k).unwrap()), v))
            })
            .collect()
    }

    pub fn in_progress(&self, block_height: u64) -> bool {
        self.status == PollStatus::InProgress && block_height <= self.end_height
    }

    pub fn load_voter(&self, storage: &dyn Storage, address: &Addr) -> StdResult<VoteInfo> {
        POLL_VOTERS.load(storage, (&self.id.to_be_bytes(), address.as_str().as_bytes()))
    }

    pub fn is_voted(&self, storage: &dyn Storage, address: &Addr) -> bool {
        self.load_voter(storage, address).is_ok()
    }

    pub fn vote(&mut self, storage: &mut dyn Storage, staker_state: &mut StakerState, vote_option: VoteOption, amount: Uint128) -> StdResult<()> {
        let vote = VoteInfo {
            voter: staker_state.address.clone(),
            option: vote_option,
            amount,
        };

        match vote.option {
            VoteOption::Yes => self.yes_votes += amount,
            VoteOption::No => self.no_votes += amount,
            VoteOption::Abstain => self.abstain_votes += amount,
        }

        POLL_VOTERS.save(storage, (&self.id.to_be_bytes(), vote.voter.as_str().as_bytes()), &vote)?;

        staker_state.vote(self.id, vote);

        Ok(())
    }

    pub fn snapshot_staked_amount(&mut self, storage: &dyn Storage, block_height: u64, contract_available_balance: Uint128) -> StdResult<Uint128> {
        let poll_config = PollConfig::load(storage)?;

        let remain_to_end: i128 = self.end_height as i128 - block_height as i128;
        if remain_to_end >= poll_config.snapshot_period as i128 {
            return Err(StdError::generic_err("Cannot snapshot at this height"));
        }

        if self.snapped_staked_amount.is_some() {
            return Err(StdError::generic_err("Snapshot has already occurred"));
        }

        self.snapped_staked_amount = Some(contract_available_balance);

        Ok(contract_available_balance)
    }

    pub fn get_vote_amount(&self) -> Uint128 {
        self.yes_votes + self.no_votes + self.abstain_votes
    }

    pub fn calculate_quorum(&self, deps: Deps, height: u64) -> StdResult<(Decimal, Uint128)> {
        let snapped_staked_amount = self.snapped_staked_amount.unwrap_or(Uint128::zero());
        let staked_amount = if snapped_staked_amount.is_zero() {
            load_available_balance(deps, height)?
        } else {
            snapped_staked_amount
        };

        Ok((
            Decimal::from_ratio(self.get_vote_amount(), staked_amount),
            staked_amount,
        ))
    }

    pub fn get_result(&self, deps: Deps, height: u64) -> StdResult<(PollResult, Uint128)> {
        let poll_config = PollConfig::load(deps.storage)?;
        let staking_state = StakingState::load(deps.storage)?;

        let votes = self.get_vote_amount();
        let (quorum, staked_amount) = if staking_state.total_share.is_zero() {
            (Decimal::zero(), Uint128::zero())
        } else {
            self.calculate_quorum(deps, height)?
        };

        if votes.is_zero() || quorum < poll_config.quorum {
            // Quorum: More than quorum of the total staked tokens at the end of the voting
            // period need to have participated in the vote.
            return Ok((PollResult::QuorumNotReached, staked_amount));
        }

        //TODO: 통과 기준이 threshold 이상인지 초과인지 확인 필요
        //Threshold: More than 50% of the tokens that participated in the vote
        let yes_ratio = Decimal::from_ratio(self.yes_votes, self.no_votes + self.yes_votes);
        if yes_ratio <= poll_config.threshold {
            return Ok((PollResult::ThresholdNotReached, staked_amount));
        }

        Ok((PollResult::Passed, staked_amount))
    }

    pub fn to_response(&self) -> PollResponse {
        PollResponse {
            id: self.id,
            title: self.title.to_string(),
            description: self.description.to_string(),
            link: self.link.clone(),
            executions: self.executions.iter().map(|v| ExecutionMsg {
                order: v.order,
                contract: v.contract.to_string(),
                msg: v.msg.clone(),
            }).collect(),
            creator: self.creator.to_string(),
            deposit_amount: self.deposit_amount,
            yes_votes: self.yes_votes,
            no_votes: self.no_votes,
            abstain_votes: self.abstain_votes,
            end_height: self.end_height,
            status: self.status.clone(),
            staked_amount: self.snapped_staked_amount,
            total_balance_at_end_poll: self.total_balance_at_end_poll,
        }
    }
}


const POLL_EXECUTION_TEMP: Item<PollExecutionContext> = Item::new("poll-execution-context");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollExecutionContext {
    pub poll_id: u64,
    pub execution_count: u64,
}

impl PollExecutionContext {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        POLL_EXECUTION_TEMP.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<PollExecutionContext> {
        POLL_EXECUTION_TEMP.load(storage)
    }

    #[cfg(test)]
    pub fn may_load(storage: &dyn Storage) -> StdResult<Option<PollExecutionContext>> {
        POLL_EXECUTION_TEMP.may_load(storage)
    }

    pub fn clear(storage: &mut dyn Storage) {
        POLL_EXECUTION_TEMP.remove(storage)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteInfo {
    pub voter: Addr,
    pub option: VoteOption,
    pub amount: Uint128,
}

#[derive(PartialEq)]
pub enum PollResult {
    Passed,
    QuorumNotReached,
    ThresholdNotReached,
}

impl fmt::Display for PollResult {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PollResult::Passed => fmt.write_str("passed"),
            PollResult::QuorumNotReached => fmt.write_str("Quorum not reached"),
            PollResult::ThresholdNotReached => fmt.write_str("Threshold not reached"),
        }
    }
}