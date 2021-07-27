use cosmwasm_std::{QuerierWrapper, StdResult, Uint128};
use terra_cosmwasm::TerraQuerier;

use crate::utils::calc_ratio_amount;

// amount = tax + send_amount
pub fn extract_tax(querier: &QuerierWrapper, denom: String, amount: Uint128) -> StdResult<Uint128> {
    if denom == "uluna" {
        return Ok(Uint128::zero());
    }

    let querier = TerraQuerier::new(querier);
    let rate = querier.query_tax_rate()?.rate;
    let tax = calc_ratio_amount(amount, rate).0;
    let cap = querier.query_tax_cap(denom)?.cap;

    Ok(std::cmp::min(tax, cap))
}

// amount = send_amount
pub fn calc_tax(querier: &QuerierWrapper, denom: String, amount: Uint128) -> StdResult<Uint128> {
    if denom == "uluna" {
        return Ok(Uint128::zero());
    }

    let querier = TerraQuerier::new(querier);
    let rate = querier.query_tax_rate()?.rate;
    let tax = Uint128::from(amount) * rate;

    let cap = querier.query_tax_cap(denom)?.cap;

    Ok(std::cmp::min(tax, cap))
}

// amount = send_amount
//extract_tax 와 비교했을 때 소수점으로 인해 1만큼의 오차가 높은 확률로 발생함.
//테스트 케이스 작성 편의를 위해 계산된 tax 에 1만큼 더한 값을 사용함.
pub fn calc_tax_one_plus(querier: &QuerierWrapper, denom: String, amount: Uint128) -> StdResult<Uint128> {
    if denom == "uluna" {
        return Ok(Uint128::zero());
    }

    let querier = TerraQuerier::new(querier);
    let rate = querier.query_tax_rate()?.rate;
    let tax = Uint128::from(amount) * rate + Uint128::from(1u128);

    let cap = querier.query_tax_cap(denom)?.cap;

    Ok(std::cmp::min(tax, cap))
}
