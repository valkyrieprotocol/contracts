use cosmwasm_std::{Deps, Env};

use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::distributor::query_msgs::{
    ContractConfigResponse, DistributorInfoResponse, DistributorInfosResponse,
};

use crate::states::{ContractConfig, DistributorInfo};

pub fn get_contract_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    Ok(ContractConfigResponse {
        governance: contract_config.governance.to_string(),
        token_contract: contract_config.token_contract.to_string(),
    })
}

pub fn get_distributor_info(
    deps: Deps,
    _env: Env,
    distributor: String,
) -> ContractResult<DistributorInfoResponse> {
    let distributor_info: DistributorInfo =
        DistributorInfo::load(deps.storage, &deps.api.addr_validate(&distributor)?)?;
    Ok(DistributorInfoResponse {
        distributor: distributor_info.distributor.to_string(),
        spend_limit: distributor_info.spend_limit,
    })
}

pub fn get_distributor_infos(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<DistributorInfosResponse> {
    Ok(DistributorInfo::query(
        deps.storage,
        start_after,
        limit,
        order_by,
    )?)
}
