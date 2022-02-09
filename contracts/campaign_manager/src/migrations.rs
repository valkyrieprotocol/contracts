use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Response};
use cw20::Denom;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::campaign_manager::execute_msgs::MigrateMsg;
use valkyrie::common::ContractResult;
use valkyrie::utils::make_response;

use crate::states::Config;

pub fn v1_0_8(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> ContractResult<Response> {
    let legacy_config = CONFIG_LEGACY.load(deps.storage)?;

    Config {
        governance: legacy_config.governance,
        valkyrie_token: legacy_config.valkyrie_token,
        vp_token: deps.api.addr_validate(msg.vp_token.as_str())?,
        terraswap_router: legacy_config.terraswap_router,
        code_id: legacy_config.code_id,
        add_pool_fee_rate: legacy_config.add_pool_fee_rate,
        add_pool_min_referral_reward_rate: legacy_config.add_pool_min_referral_reward_rate,
        remove_pool_fee_rate: legacy_config.remove_pool_fee_rate,
        fee_burn_ratio: legacy_config.fee_burn_ratio,
        fee_recipient: legacy_config.fee_recipient,
        deactivate_period: legacy_config.deactivate_period,
        key_denom: legacy_config.key_denom,
        contract_admin: legacy_config.contract_admin,
    }.save(deps.storage)?;

    Ok(make_response("migrate_v1_0_8"))
}

const CONFIG_LEGACY: Item<LegacyConfig> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub governance: Addr,
    pub valkyrie_token: Addr,
    pub terraswap_router: Addr,
    pub code_id: u64,
    pub add_pool_fee_rate: Decimal,
    pub add_pool_min_referral_reward_rate: Decimal,
    pub remove_pool_fee_rate: Decimal,
    pub fee_burn_ratio: Decimal,
    pub fee_recipient: Addr,
    pub deactivate_period: u64,
    pub key_denom: Denom,
    pub contract_admin: Addr,
}