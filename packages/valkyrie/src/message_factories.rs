use cosmwasm_std::{Addr, CosmosMsg, to_binary, Uint128, WasmMsg, BankMsg, Coin, Binary};
use cw20::Cw20ExecuteMsg;

pub fn native_send(denom: String, recipient: &Addr, amount: Uint128) -> CosmosMsg {
    CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom,
            amount,
        }],
    })
}

pub fn cw20_transfer(token: &Addr, recipient: &Addr, amount: Uint128) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        send: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        }).unwrap(),
    })
}

pub fn wasm_instantiate(
    code_id: u64,
    admin: Option<Addr>,
    msg: Binary,
) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: admin.map(|v| v.to_string()),
        code_id,
        msg,
        send: vec![],
        label: String::new(),
    })
}