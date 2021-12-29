use cw_storage_plus::{Item, Map};
use cosmwasm_std::{Addr, Storage, StdResult, Uint128, Order, StdError, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");
const ADMIN_NOMINEE: Item<Addr> = Item::new("admin_nominee");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: Addr,
    pub managing_token: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn may_load_admin_nominee(storage: &dyn Storage) -> StdResult<Option<Addr>> {
        ADMIN_NOMINEE.may_load(storage)
    }

    pub fn save_admin_nominee(storage: &mut dyn Storage, address: &Addr) -> StdResult<()> {
        ADMIN_NOMINEE.save(storage, address)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admin == *address
    }
}


const CONTRACT_STATE: Item<ContractState> = Item::new("contract-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractState {
    pub distribution_count: u64,
    pub locked_amount: Uint128,
    pub distributed_amount: Uint128,
}

impl ContractState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractState> {
        CONTRACT_STATE.load(storage)
    }

    pub fn lock(&mut self, balance: Uint128, amount: Uint128) -> StdResult<Uint128> {
        self.locked_amount += amount;

        balance.checked_sub(self.locked_amount)
            .map_err(|e| StdError::overflow(e))
    }

    pub fn unlock(&mut self, amount: Uint128) -> StdResult<()> {
        self.locked_amount = self.locked_amount.checked_sub(amount)
            .map_err(|e| StdError::overflow(e))?;

        Ok(())
    }
}


const DISTRIBUTIONS: Map<&[u8], Distribution> = Map::new("distribution");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Distribution {
    pub id: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub recipient: Addr,
    pub amount: Uint128,
    pub distributed_amount: Uint128,
    pub message: Option<Binary>,
}

impl Distribution {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        if self.end_height < self.start_height {
            return Err(StdError::generic_err("start_height must be less than end_height"));
        }

        DISTRIBUTIONS.save(storage, &self.id.to_be_bytes(), self)
    }

    pub fn may_load(storage: &dyn Storage, id: u64) -> StdResult<Option<Distribution>> {
        DISTRIBUTIONS.may_load(storage, &id.to_be_bytes())
    }

    pub fn load_all(storage: &dyn Storage) -> StdResult<Vec<Distribution>> {
        DISTRIBUTIONS.range(storage, None, None, Order::Ascending)
            .map(|d| Ok(d?.1))
            .collect()
    }

    pub fn delete(&self, storage: &mut dyn Storage) {
        DISTRIBUTIONS.remove(storage, &self.id.to_be_bytes())
    }

    pub fn released_amount(&self, height: u64) -> Uint128 {
        if self.start_height > height {
            return Uint128::zero();
        }

        let released_amount = self.amount.multiply_ratio(
            height - self.start_height,
            self.end_height - self.start_height,
        );

        std::cmp::min(released_amount, self.amount)
    }
}
