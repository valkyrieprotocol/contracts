use cosmwasm_std::{Addr, attr, Attribute, QuerierWrapper, QueryRequest, StdResult, to_binary, Uint128, WasmQuery};
use cw20::{Denom, Cw20QueryMsg, TokenInfoResponse};

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

pub fn query_token_info(
    querier: &QuerierWrapper,
    contract_addr: String,
) -> StdResult<TokenInfoResponse> {
    let token_info: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr,
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(token_info)
}