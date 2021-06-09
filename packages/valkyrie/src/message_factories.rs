use cosmwasm_std::{Addr, CosmosMsg, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

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