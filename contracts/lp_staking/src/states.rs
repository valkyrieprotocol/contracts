use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

pub const UST: &str = "uusd";

const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub token: Addr,
    pub lp_token: Addr,
    pub pair: Addr,
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        CONFIG.load(storage)
    }
}

const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub pending_reward: Uint128, // not distributed amount due to zero bonding
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal,
}

impl State {
    pub fn default() -> State {
        State {
            pending_reward: Uint128::zero(),
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<State> {
        Ok(STATE.may_load(storage)?.unwrap_or_else(|| State::default()))
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
}
