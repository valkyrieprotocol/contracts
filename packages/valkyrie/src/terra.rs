use cosmwasm_std::{QuerierWrapper, StdResult};
use terra_cosmwasm::TerraQuerier;
use crate::utils::calc_ratio_amount;

// amount = tax + send_amount
pub fn extract_tax(querier: &QuerierWrapper, denom: String, amount: u128) -> StdResult<u128> {
    if denom == "uluna" {
        return Ok(0u128);
    }

    let querier = TerraQuerier::new(querier);
    let rate = querier.query_tax_rate()?.rate;
    let tax = calc_ratio_amount(amount, rate).0;
    let cap = querier.query_tax_cap(denom)?.cap.u128();

    Ok(std::cmp::min(tax, cap))
}
