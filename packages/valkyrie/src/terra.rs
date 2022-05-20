use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use terra_cosmwasm::TerraQuerier;

pub fn is_contract(querier: &QuerierWrapper, address: &Addr) -> StdResult<bool> {
    let querier = TerraQuerier::new(querier);

    Ok(querier.query_contract_info(address.to_string())
        .map_or(false, |_| true))
}
