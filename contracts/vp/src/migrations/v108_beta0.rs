use cosmwasm_std::{Addr, Decimal, Env, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Config, SwapRatio};

pub fn migrate(
    storage: &mut dyn Storage,
    _env: &Env,
    router: &Addr,
) -> StdResult<()> {
    let legacy_config = ConfigV108Beta0::load(storage)?;

    Config {
        admin: legacy_config.admin,
        whitelist: legacy_config.whitelist,
        offer_token: legacy_config.offer_token,
        base_swap_ratio: legacy_config.base_swap_ratio,
        custom_swap_ratio: legacy_config.custom_swap_ratio,
        router: router.clone(),
    }.save(storage)?;

    Ok(())
}

const CONFIG_V108_BETA0: Item<ConfigV108Beta0> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigV108Beta0 {
    pub admin: Addr,
    pub whitelist: Vec<Addr>,
    pub offer_token: Addr,
    pub base_swap_ratio: Decimal,
    pub custom_swap_ratio: Vec<SwapRatio>,
}

impl ConfigV108Beta0 {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG_V108_BETA0.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ConfigV108Beta0> {
        CONFIG_V108_BETA0.load(storage)
    }
}