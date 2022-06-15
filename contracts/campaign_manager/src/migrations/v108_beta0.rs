// use cosmwasm_std::{Addr, Decimal, Env, StdResult, Storage};
// use cw20::Denom;
// use cw_storage_plus::Item;
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
// use crate::states::Config;
//
// pub fn migrate(
//     storage: &mut dyn Storage,
//     _env: &Env,
//     vp_token: &Addr,
// ) -> StdResult<()> {
//     let legacy_config = ConfigV108Beta0::load(storage)?;
//
//     Config {
//         governance: legacy_config.governance,
//         valkyrie_token: legacy_config.valkyrie_token,
//         vp_token: vp_token.clone(),
//         valkyrie_proxy: legacy_config.valkyrie_proxy,
//         code_id: legacy_config.code_id,
//         add_pool_fee_rate: legacy_config.add_pool_fee_rate,
//         add_pool_min_referral_reward_rate: legacy_config.add_pool_min_referral_reward_rate,
//         remove_pool_fee_rate: legacy_config.remove_pool_fee_rate,
//         fee_burn_ratio: legacy_config.fee_burn_ratio,
//         fee_recipient: legacy_config.fee_recipient,
//         deactivate_period: legacy_config.deactivate_period,
//         key_denom: legacy_config.key_denom,
//         contract_admin: legacy_config.contract_admin,
//     }.save(storage)?;
//
//     Ok(())
// }
//
// const CONFIG_V108_BETA0: Item<ConfigV108Beta0> = Item::new("config");
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct ConfigV108Beta0 {
//     pub governance: Addr,
//     pub valkyrie_token: Addr,
//     pub valkyrie_proxy: Addr,
//     pub code_id: u64,
//     pub add_pool_fee_rate: Decimal,
//     pub add_pool_min_referral_reward_rate: Decimal,
//     pub remove_pool_fee_rate: Decimal,
//     pub fee_burn_ratio: Decimal,
//     pub fee_recipient: Addr,
//     pub deactivate_period: u64,
//     pub key_denom: Denom,
//     pub contract_admin: Addr,
// }
//
// impl ConfigV108Beta0 {
//     pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
//         CONFIG_V108_BETA0.save(storage, self)
//     }
//
//     pub fn load(storage: &dyn Storage) -> StdResult<ConfigV108Beta0> {
//         CONFIG_V108_BETA0.load(storage)
//     }
// }