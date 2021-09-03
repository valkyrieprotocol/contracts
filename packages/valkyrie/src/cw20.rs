use cosmwasm_std::{Addr, attr, Attribute, QuerierWrapper, StdResult, Uint128};
use cw20::{Denom, Cw20QueryMsg};

pub fn query_balance(
    querier: &QuerierWrapper,
    denom: Denom,
    address: Addr,
) -> StdResult<Uint128> {
    match denom {
        Denom::Native(denom) => querier
            .query_balance(address, denom)
            .map(|v| v.amount),
        Denom::Cw20(contract_addr) => {
            query_cw20_balance(querier, &contract_addr, &address)
        }
    }
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    let response: cw20::BalanceResponse = querier.query_wasm_smart(
        contract_addr,
        &Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        },
    )?;

    Ok(response.balance)
}

pub fn create_send_attr(recipient: &Addr, amount: Uint128, action: &str) -> Vec<Attribute> {
    vec![
        attr("action", action),
        attr("recipient", recipient.as_str()),
        attr("amount", amount.to_string()),
    ]
}
