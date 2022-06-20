use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use valkyrie::utils::is_contract;

const CONFIG: Item<Config> = Item::new("config_v3");
const ADMIN_NOMINEE: Item<Addr> = Item::new("admin_nominee");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub token: Addr,
    pub usdc_token: Addr,
    pub lp_token: Addr,
    pub pair: Addr,
    pub whitelisted_contracts: Vec<Addr>,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)?;
        Ok(())
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        CONFIG.load(storage)
    }

    pub fn is_whitelisted_contract(&self, address: &Addr) -> bool {
        self.whitelisted_contracts.contains(&address)
    }

    pub fn is_authorized(&self, address: &Addr) -> StdResult<bool> {
        if is_contract(&address) && !self.is_whitelisted_contract(&address) {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn may_load_admin_nominee(storage: &dyn Storage) -> StdResult<Option<Addr>> {
        ADMIN_NOMINEE.may_load(storage)
    }

    pub fn save_admin_nominee(storage: &mut dyn Storage, address: &Addr) -> StdResult<()> {
        ADMIN_NOMINEE.save(storage, address)
    }
}

const STATE: Item<State> = Item::new("state_v2");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal,
}

impl State {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<State> {
        STATE.load(storage)
    }

    // compute distributed rewards and update global reward index
    pub fn compute_reward(&mut self, config: &Config, block_height: u64) {
        if self.total_bond_amount.is_zero() {
            self.last_distributed = block_height;
            return;
        }

        let mut distributed_amount: Uint128 = Uint128::zero();
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
            + Decimal::from_ratio(distributed_amount, self.total_bond_amount);
        // state.global_reward_index = state.global_reward_index + (distributed_amount / state.total_bond_amount)
    }
}

const STAKER_INFO: Map<&str, StakerInfo> = Map::new("reward");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub owner: Addr,
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
}

impl StakerInfo {
    pub fn default(owner: Addr) -> StakerInfo {
        StakerInfo {
            owner,
            reward_index: Decimal::zero(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKER_INFO.save(storage, self.owner.as_str(), self)
    }

    pub fn load_or_default(storage: &dyn Storage, owner: &Addr) -> StdResult<StakerInfo> {
        Ok(STAKER_INFO
            .may_load(storage, owner.as_str())?
            .unwrap_or_else(|| StakerInfo::default(owner.clone())))
    }

    pub fn delete(&self, storage: &mut dyn Storage) {
        STAKER_INFO.remove(storage, self.owner.as_str())
    }

    // withdraw reward to pending reward
    pub fn compute_staker_reward(&mut self, state: &State) -> StdResult<()> {
        let pending_reward = (self.bond_amount * state.global_reward_index)
            .checked_sub(self.bond_amount * self.reward_index)?;

        self.reward_index = state.global_reward_index;
        self.pending_reward += pending_reward;
        Ok(())
    }
}
