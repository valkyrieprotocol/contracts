use cosmwasm_std::{Addr, Api, attr, Attribute, Binary, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery};
use cw20::Denom;

pub fn query_balance(
    querier: &QuerierWrapper,
    api: &dyn Api,
    denom: Denom,
    address: Addr,
) -> StdResult<Uint128> {
    match denom {
        Denom::Native(denom) => querier
            .query_balance(address, denom)
            .map(|v| v.amount),
        Denom::Cw20(contract_addr) => {
            query_cw20_balance(querier, api, &contract_addr, &address)
        }
    }
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    api: &dyn Api,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    Ok(querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr.to_string(),
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                (api.addr_canonicalize(account_addr.as_str())?).as_slice(),
            )),
        }))
        .unwrap_or_else(|_| Uint128::zero()))
}

pub fn create_send_attr(recipient: &Addr, amount: Uint128, action: &str) -> Vec<Attribute> {
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

// Copy from cosmwasm-storage v0.14.1
fn to_length_prefixed(namespace: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(namespace.len() + 2);
    out.extend_from_slice(&encode_length(namespace));
    out.extend_from_slice(namespace);
    out
}

// Copy from cosmwasm-storage v0.14.1
fn encode_length(namespace: &[u8]) -> [u8; 2] {
    if namespace.len() > 0xFFFF {
        panic!("only supports namespaces up to length 0xFFFF")
    }
    let length_bytes = (namespace.len() as u32).to_be_bytes();
    [length_bytes[2], length_bytes[3]]
}