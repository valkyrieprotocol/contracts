// use cosmwasm_std::{Addr, Decimal, Env, StdResult, Storage};
// use cw20::Denom;
// use cw_storage_plus::Item;
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
//
// use crate::migrations::v108_beta0::ConfigV108Beta0;
//
// pub fn migrate(
//     storage: &mut dyn Storage,
//     _env: &Env,
//     contract_admin: &Addr,
// ) -> StdResult<()> {
//     let legacy_config = ConfigV106::load(storage)?;
//
//     ConfigV108Beta0 {
//         governance: legacy_config.governance,
//         valkyrie_token: legacy_config.valkyrie_token,
//         valkyrie_proxy: legacy_config.valkyrie_proxy,
//         code_id: legacy_config.code_id,
//         add_pool_fee_rate: legacy_config.add_pool_fee_rate,
//         add_pool_min_referral_reward_rate: legacy_config.add_pool_min_referral_reward_rate,
//         remove_pool_fee_rate: legacy_config.remove_pool_fee_rate,
//         fee_burn_ratio: legacy_config.fee_burn_ratio,
//         fee_recipient: legacy_config.fee_recipient,
//         deactivate_period: legacy_config.deactivate_period,
//         key_denom: legacy_config.key_denom,
//         contract_admin: contract_admin.clone(),
//     }.save(storage)?;
//
//     Ok(())
// }
//
// const CONFIG_V106: Item<ConfigV106> = Item::new("config");
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct ConfigV106 {
//     pub governance: Addr,
//     pub valkyrie_token: Addr,
//     pub terraswap_router: Addr,
//     pub code_id: u64,
//     pub add_pool_fee_rate: Decimal,
//     pub add_pool_min_referral_reward_rate: Decimal,
//     pub remove_pool_fee_rate: Decimal,
//     pub fee_burn_ratio: Decimal,
//     pub fee_recipient: Addr,
//     pub deactivate_period: u64,
//     pub key_denom: Denom,
// }
//
// impl ConfigV106 {
//     pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
//         CONFIG_V106.save(storage, self)
//     }
//
//     pub fn load(storage: &dyn Storage) -> StdResult<ConfigV106> {
//         CONFIG_V106.load(storage)
//     }
// }