use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin: Addr,
    pub whitelist: Vec<Addr>,
    pub offer_token: Addr,
    pub base_swap_ratio: Decimal,
    pub custom_swap_ratio: Vec<SwapRatio>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SwapRatio {
    pub address: Addr,
    pub ratio: Decimal
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        &self.admin == address
    }

    pub fn is_whitelisted(&self, address: &Addr) -> bool {
        self.whitelist.contains(address)
    }

    pub fn get_swap_ratio(&self, address: &Addr) -> Decimal {
        self.custom_swap_ratio.iter()
            .find(|it| it.address == *address)
            .map(|item| item.ratio)
            .unwrap_or(self.base_swap_ratio)
    }
}

const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct State {
    pub cumulative_offer_amount: Uint128,
    pub cumulative_mint_amount: Uint128,
}

impl State {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STATE.save(storage, self)
    }

    pub fn load_or_default(storage: &dyn Storage) -> StdResult<State> {
        Ok(STATE
            .may_load(storage)?
            .unwrap_or_else(|| State {
                cumulative_offer_amount: Uint128::zero(),
                cumulative_mint_amount: Uint128::zero()
            }))
    }
}

const SWAP_STATE: Map<&str, SwapState> = Map::new("swap_state");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SwapState {
    pub ratio:Decimal,
    pub cumulative_offer_amount:Uint128
}

impl SwapState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        SWAP_STATE.save(storage, self.ratio.to_string().as_str(), self)
    }

    pub fn load_or_default(storage: &dyn Storage, ratio:Decimal) -> StdResult<SwapState> {
        Ok(SWAP_STATE
            .may_load(storage, ratio.to_string().as_str())?
            .unwrap_or_else(|| SwapState {
                ratio,
                cumulative_offer_amount: Uint128::zero()
            }))
    }
}