use crate::terra::extract_tax;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, QuerierWrapper, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

pub fn native_send(
    querier: &QuerierWrapper,
    denom: String,
    recipient: &Addr,
    amount_with_tax: Uint128,
) -> StdResult<CosmosMsg> {
    let tax = extract_tax(querier, denom.to_string(), amount_with_tax)?;

    Ok(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            amount: amount_with_tax.checked_sub(tax)?,
            denom,
        }],
    }))
}

pub fn cw20_transfer(token: &Addr, recipient: &Addr, amount: Uint128) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        send: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        })
        .unwrap(),
    })
}

pub fn wasm_instantiate(code_id: u64, admin: Option<Addr>, msg: Binary) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: admin.map(|v| v.to_string()),
        code_id,
        msg,
        send: vec![],
        label: String::new(),
    })
}
