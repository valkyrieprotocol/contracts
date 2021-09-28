use cosmwasm_std::{Deps, Env};

use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::fund_manager::query_msgs::{AllowanceResponse, AllowancesResponse, BalanceResponse, ContractConfigResponse};

use crate::states::{Allowance, ContractConfig, ContractState};

pub fn get_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let config = ContractConfig::load(deps.storage)?;

    Ok(ContractConfigResponse {
        admins: config.admins.iter().map(|v| v.to_string()).collect(),
        managing_token: config.managing_token.to_string(),
    })
}

pub fn get_balance(deps: Deps, env: Env) -> ContractResult<BalanceResponse> {
    let config = ContractConfig::load(deps.storage)?;
    let state = ContractState::load(deps.storage)?;

    Ok(state.load_balance(&deps.querier, &env, &config.managing_token)?)
}

pub fn get_allowance(deps: Deps, _env: Env, address: String) -> ContractResult<AllowanceResponse> {
    let address = deps.api.addr_validate(address.as_str())?;
    let allowance = Allowance::load(deps.storage, &address)?;

    Ok(AllowanceResponse {
        address: address.to_string(),
        allowed_amount: allowance.allowed_amount,
        remain_amount: allowance.remain_amount,
    })
}

pub fn query_allowances(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<AllowancesResponse> {
    let allowances = Allowance::query(
        deps.storage,
        start_after,
        limit,
        order_by,
    )?;

    Ok(allowances)
}
