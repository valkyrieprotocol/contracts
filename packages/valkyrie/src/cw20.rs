use cosmwasm_std::{Addr, Api, attr, Attribute, Binary, CanonicalAddr, CosmosMsg, QuerierWrapper, QueryRequest, Response, StdResult, to_binary, Uint128, WasmMsg, WasmQuery};
use cosmwasm_storage::to_length_prefixed;
use cw20::Cw20ExecuteMsg;

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    api: &dyn Api,
    contract_addr: &CanonicalAddr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    Ok(
        querier.query(
            &QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: api.addr_humanize(contract_addr)?.to_string(),
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
    amount: u128,
    action: &str,
) -> Response {
    Response {
        submessages: vec![],
        messages: create_send_msg(token, recipient, amount),
        attributes: create_send_attr(recipient, amount, action),
        data: None,
    }
}

pub fn create_send_msg(
    token: &Addr,
    recipient: &Addr,
    amount: u128,
) -> Vec<CosmosMsg> {
    vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::Transfer {
                    recipient: recipient.to_string(),
                    amount: Uint128::from(amount),
                }
            ).unwrap(),
            send: vec![],
        })
    ]
}

pub fn create_send_attr(
    recipient: &Addr,
    amount: u128,
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
