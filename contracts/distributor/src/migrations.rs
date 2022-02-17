use cosmwasm_std::{Addr, Env, StdResult, Storage};
#[cfg(not(feature = "library"))]
use cw_storage_plus::Item;
use crate::states::ContractConfig;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub fn migrate(
    storage: &mut dyn Storage,
    _env: &Env,
) -> StdResult<()> {
    let legacy_config = LagacyConfig::load(storage)?;

    ContractConfig {
        admin: legacy_config.admins[0].clone(),
        managing_token: legacy_config.managing_token
    }.save(storage)?;

    Ok(())
}

const LAGACY_CONFIG: Item<LagacyConfig> = Item::new("contract-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LagacyConfig {
    pub admins: Vec<Addr>,
    pub managing_token: Addr,
}

impl LagacyConfig {
    pub fn load(storage: &dyn Storage) -> StdResult<LagacyConfig> {
        LAGACY_CONFIG.load(storage)
    }
}