use cosmwasm_std::{CosmosMsg, WasmMsg, from_binary, Uint128};
use cw20::Cw20ExecuteMsg;

pub struct Cw20Transfer {
    pub contract_addr: String,
    pub recipient: String,
    pub amount: Uint128,
}

pub fn cw20_transfer(msgs: &Vec<CosmosMsg>) -> Vec<Cw20Transfer> {
    let mut result = vec![];

    for msg in msgs.iter() {
        match msg {
            CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, send: _, msg }) => {
                match from_binary(msg).unwrap() {
                    Cw20ExecuteMsg::Transfer { recipient, amount } => {
                        result.push(Cw20Transfer {
                            contract_addr: contract_addr.to_string(),
                            recipient,
                            amount,
                        });
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    result
}