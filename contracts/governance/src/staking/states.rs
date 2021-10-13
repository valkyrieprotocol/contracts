use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::governance::enumerations::PollStatus;

use crate::poll::states::{Poll, VoteInfo};


const STAKING_CONFIG: Item<StakingConfig> = Item::new("staking-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingConfig {
    pub distributor: Option<Addr>,
}

impl StakingConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKING_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<StakingConfig> {
        STAKING_CONFIG.load(storage)
    }
}

const STAKING_STATE: Item<StakingState> = Item::new("staking-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingState {
    pub total_share: Uint128,
}

impl StakingState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKING_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<StakingState> {
        STAKING_STATE.load(storage)
    }
}


const STAKER_STATES: Map<&Addr, StakerState> = Map::new("staker-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerState {
    pub address: Addr,
    pub share: Uint128,
    // total staked balance
    pub votes: Vec<(u64, VoteInfo)>, // maps poll_id to weight voted
}

impl StakerState {
    pub fn default(address: &Addr) -> StakerState {
        StakerState {
            address: address.clone(),
            share: Uint128::zero(),
            votes: vec![],
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKER_STATES.save(storage, &self.address, self)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<StakerState> {
        STAKER_STATES.load(storage, address)
    }

    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<StakerState>> {
        STAKER_STATES.may_load(storage, address)
    }

    pub fn load_safe(storage: &dyn Storage, address: &Addr) -> StdResult<StakerState> {
        Ok(STAKER_STATES.may_load(storage, address)?.unwrap_or(StakerState::default(address)))
    }

    pub fn clean_votes(&mut self, storage: &dyn Storage) -> () {
        self.votes.retain(|(poll_id, _)| {
            Poll::load(storage, &poll_id).ok()
                .map(|p| p.status == PollStatus::InProgress)
                .unwrap_or(false)
        });
    }

    pub fn load_balance(&self, storage: &dyn Storage, contract_available_balance: Uint128) -> StdResult<Uint128> {
        let staking_state = StakingState::load(storage)?;

        if staking_state.total_share.is_zero() {
            return Ok(Uint128::zero())
        }

        let staker_balance = self.share.multiply_ratio(
            contract_available_balance,
            staking_state.total_share,
        );

        return Ok(staker_balance);
    }

    // removes not in-progress poll voter info & unlock tokens
    // and returns the largest locked amount in participated polls.
    pub fn get_locked_balance(&self) -> Uint128 {
        self.votes.iter()
            .map(|(_, v)| v.amount)
            .max()
            .unwrap_or_default()
    }

    pub fn can_vote(&self, storage: &dyn Storage, contract_available_balance: Uint128, amount: Uint128) -> StdResult<bool> {
        let balance = self.load_balance(storage, contract_available_balance)?;

        Ok(balance >= amount)
    }

    pub fn vote(&mut self, poll_id: u64, vote: VoteInfo) {
        self.votes.push((poll_id, vote));
    }
}
