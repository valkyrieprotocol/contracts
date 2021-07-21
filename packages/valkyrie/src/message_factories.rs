use cosmwasm_std::{to_binary, Addr, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

pub fn cw20_transfer(token: &Addr, recipient: &Addr, amount: Uint128) -> SubMsg {
    SubMsg::new(WasmMsg::Execute {
        contract_addr: token.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        })
        .unwrap(),
    })
}
