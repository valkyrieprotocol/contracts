use cosmwasm_std::{Deps, Env};

use valkyrie::common::ContractResult;
use valkyrie::governance::query_msgs::ValkyrieConfigResponse;

use super::states::ValkyrieConfig;

pub fn get_valkyrie_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<ValkyrieConfigResponse> {
    let valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    Ok(
        ValkyrieConfigResponse {
            burn_contract: valkyrie_config.burn_contract.to_string(),
            reward_withdraw_burn_rate: valkyrie_config.reward_withdraw_burn_rate,
        }
    )
}
