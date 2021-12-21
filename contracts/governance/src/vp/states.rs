use cosmwasm_std::{Addr, Decimal, Deps, Env, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::staking::states::{StakerState, StakingState};


const TICKET_CONFIG: Item<TicketConfig> = Item::new("vp-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TicketConfig {
    pub ticket_token: Addr,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

impl TicketConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        TICKET_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<TicketConfig> {
        TICKET_CONFIG.load(storage)
    }
}

const TICKET_STATE: Item<TicketState> = Item::new("vp-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TicketState {
    pub last_distributed: u64,
    pub global_reward_index: Decimal,
}

impl TicketState {
    pub fn default() -> TicketState {
        TicketState {
            last_distributed: 0,
            global_reward_index: Decimal::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        TICKET_STATE.save(storage, self)
    }

    pub fn load_or_default(storage: &dyn Storage) -> TicketState {
        TICKET_STATE.load(storage).unwrap_or(TicketState::default())
    }

    pub fn compute_distributed_reward(&mut self, deps:&Deps, block_height:u64) -> StdResult<()> {
        let staking_state = StakingState::load(deps.storage)?;
        if staking_state.total_share.is_zero() {
            self.last_distributed = block_height;
            return Ok(());
        }

        let config = TicketConfig::load(deps.storage)?;
        let mut distributed_amount = Uint128::zero();
        for s in config.distribution_schedule.iter() {
            //s.0 = begin block height of this schedule
            //s.1 = end block height of this schedule
            if s.0 > block_height || s.1 < self.last_distributed {
                continue;
            }

            // min(s.1, block_height) - max(s.0, last_distributed)
            let passed_blocks =
                std::cmp::min(s.1, block_height) - std::cmp::max(s.0, self.last_distributed);

            let num_blocks = s.1 - s.0;
            let distribution_amount_per_block: Decimal = Decimal::from_ratio(s.2, num_blocks);
            // distribution_amount_per_block = distribution amount of this schedule / blocks count of this schedule.
            distributed_amount +=
                distribution_amount_per_block * Uint128::new(passed_blocks as u128);
        }

        self.last_distributed = block_height;
        self.global_reward_index = self.global_reward_index
            + Decimal::from_ratio(distributed_amount, staking_state.total_share);

        return Ok(())
    }
}

const TICKET_STAKER_STATE: Map<&Addr, TicketStakerState> = Map::new("vp-staker-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TicketStakerState {
    pub address: Addr,
    pub reward_index: Decimal,
    pub pending_reward: Uint128,
}

impl TicketStakerState {
    pub fn default(address: &Addr) -> TicketStakerState {
        TicketStakerState {
            address: address.clone(),
            reward_index: Decimal::zero(),
            pending_reward: Uint128::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        TICKET_STAKER_STATE.save(storage, &self.address, self)
    }

    pub fn load_or_default(storage: &dyn Storage, address: &Addr) -> TicketStakerState {
        TICKET_STAKER_STATE.load(storage, address).unwrap_or(TicketStakerState::default(&address))
    }

    pub fn delete(&self, storage: &mut dyn Storage) -> () {
        TICKET_STAKER_STATE.remove(storage, &self.address)
    }

    pub fn compute_staker_reward(&mut self, staker_state:&StakerState, ticket_state:&TicketState) -> StdResult<()> {
        let pending_reward = (staker_state.share * ticket_state.global_reward_index)
            .checked_sub(staker_state.share * self.reward_index)?;

        self.reward_index = ticket_state.global_reward_index;
        self.pending_reward += pending_reward;
        Ok(())
    }
}

pub fn compute_ticket(
    deps: &Deps,
    env: &Env,
    staker: &Addr,
) -> StdResult<(TicketState, TicketStakerState)> {
    let staker_state = StakerState::load_safe(deps.storage, &staker)?;
    let mut ticket_state = TicketState::load_or_default(deps.storage);
    ticket_state.compute_distributed_reward(deps, env.block.height)?;

    let mut ticket_staker_state = TicketStakerState::load_or_default(deps.storage, &staker);
    ticket_staker_state.compute_staker_reward(&staker_state, &ticket_state)?;

    Ok((ticket_state, ticket_staker_state))
}

