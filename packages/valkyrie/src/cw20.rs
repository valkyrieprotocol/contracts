use cosmwasm_std::{Addr, Api, attr, Attribute, Binary, QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmQuery};
use cosmwasm_storage::to_length_prefixed;
use crate::message_factories;
use cw20::Denom;

pub fn query_balance(
    querier: &QuerierWrapper,
    api: &dyn Api,
    denom: Denom,
    address: Addr,
) -> StdResult<u128> {
    match denom {
        Denom::Native(denom) => querier.query_balance(address, denom).map(|v| {
            v.amount.u128()
        }),
        Denom::Cw20(contract_addr) => query_cw20_balance(
            querier,
            api,
            &contract_addr,
            &address,
        ).map(|v| v.u128()),
    }
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    api: &dyn Api,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    Ok(
        querier.query(
            &QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: contract_addr.to_string(),
                key: Binary::from(concat(
                    &to_length_prefixed(b"balance").to_vec(),
                    (api.addr_canonicalize(account_addr.as_str())?).as_slice(),
                )),
            })
        ).unwrap_or_else(|_| Uint128::zero())
    )
}

pub fn create_send_msg_response(
    token: &Addr,
    recipient: &Addr,
    amount: Uint128,
    action: &str,
) -> Response {
    Response {
        submessages: vec![],
        messages: vec![message_factories::cw20_transfer(token, recipient, amount.clone())],
        attributes: create_send_attr(recipient, amount, action),
        data: None,
    }
}

pub fn create_send_attr(
    recipient: &Addr,
    amount: Uint128,
    action: &str,
) -> Vec<Attribute> {
    vec![
        attr("action", action),
        attr("recipient", recipient.as_str()),
        attr("amount", amount.to_string()),
    ]
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
