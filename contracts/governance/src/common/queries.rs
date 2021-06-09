use cosmwasm_std::{Deps, Env};

use valkyrie::common::ContractResult;
use valkyrie::governance::models::ContractConfigResponse;

use super::states::ContractConfig;

pub fn get_contract_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<ContractConfigResponse> {
    let contract_config = ContractConfig::load(deps.storage)?;

    Ok(
        ContractConfigResponse {
            token_contract: contract_config.token_contract,
        }
    )
}