use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Deps, StdResult, Uint128};
use cw_storage_plus::{Item, Map};

pub const UST: &str = "uusd";

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const STAKER_INFO: Map<&[u8], StakerInfo> = Map::new("reward");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub valkyrie_token: Addr,
    pub liquidity_token: Addr,
    pub pair_contract: Addr,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal, // 누적 분배 비율, 4년간 분배될 것중 얼마나 분배가 됐나.
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
}

pub fn read_staker_info(deps: &Deps, owner: &Addr) -> StdResult<StakerInfo> {
    match STAKER_INFO.may_load(
        deps.storage,
        deps.api.addr_canonicalize(owner.as_str())?.as_slice(),
    )? {
        Some(staker_info) => Ok(staker_info),
        None => Ok(StakerInfo {
            reward_index: Decimal::zero(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
        }),
    }
}
