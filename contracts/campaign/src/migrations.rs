use cosmwasm_std::{Addr, DepsMut, Env, Response, Uint128};
use cw20::Denom;
use cw_storage_plus::Item;
use valkyrie::campaign::execute_msgs::{MigrateMsg};
use valkyrie::common::{ContractResult};
use valkyrie::utils::{make_response};
use crate::executions::validate_participation_reward_distribution_schedule;

use crate::states::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("migrate");

    validate_participation_reward_distribution_schedule(&msg.participation_reward_distribution_schedule)?;

    let old_reward_config = OLD_REWARD_CONFIG.load(deps.storage)?;
    RewardConfig {
        participation_reward_denom: old_reward_config.participation_reward_denom,
        participation_reward_amount: old_reward_config.participation_reward_amount,
        participation_reward_lock_period: old_reward_config.participation_reward_lock_period,
        participation_reward_distribution_schedule: msg.participation_reward_distribution_schedule,
        referral_reward_token: old_reward_config.referral_reward_token,
        referral_reward_amounts: old_reward_config.referral_reward_amounts,
        referral_reward_lock_period: old_reward_config.referral_reward_lock_period,
    }.save(deps.storage)?;

    Ok(response)
}

pub const OLD_REWARD_CONFIG: Item<OldRewardConfig> = Item::new("reward_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OldRewardConfig {
    pub participation_reward_denom: Denom,
    pub participation_reward_amount: Uint128,
    pub participation_reward_lock_period: u64,
    pub referral_reward_token: Addr,
    pub referral_reward_amounts: Vec<Uint128>,
    pub referral_reward_lock_period: u64,
}