use cosmwasm_std::{Addr, StdResult, Storage, Decimal};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


const VALKYRIE_CONFIG: Item<ValkyrieConfig> = Item::new("valkyrie-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValkyrieConfig {
    pub burn_contract: Addr,
    pub reward_withdraw_burn_rate: Decimal,
}

impl ValkyrieConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        VALKYRIE_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ValkyrieConfig> {
        VALKYRIE_CONFIG.load(storage)
    }
}
