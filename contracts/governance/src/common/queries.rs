use cosmwasm_std::{Deps, Env};

use valkyrie::common::ContractResult;
use valkyrie::governance::query_msgs::ContractConfigResponse;

use super::states::ContractConfig;

pub fn get_contract_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<ContractConfigResponse> {
    let contract_config = ContractConfig::load(deps.storage)?;

    Ok(
        ContractConfigResponse {
            governance_token: contract_config.governance_token.to_string(),
        }
    )
}