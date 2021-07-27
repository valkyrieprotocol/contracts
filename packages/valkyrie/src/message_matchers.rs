use cosmwasm_std::{BankMsg, Coin, CosmosMsg, from_binary, Uint128, WasmMsg, SubMsg};
use cw20::Cw20ExecuteMsg;

pub struct NativeSend {
    pub to_address: String,
    pub amount: Vec<Coin>,
}

pub fn native_send(msgs: &Vec<SubMsg>) -> Vec<NativeSend> {
    let mut result = vec![];

    for msg in msgs.iter().map(|m| &m.msg) {
        match msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                result.push(NativeSend {
                    to_address: to_address.clone(),
                    amount: amount.clone(),
                });
            }
            _ => {}
        }
    }

    result
}

pub struct Cw20Transfer {
    pub contract_addr: String,
    pub recipient: String,
    pub amount: Uint128,
}

pub fn cw20_transfer(msgs: &Vec<SubMsg>) -> Vec<Cw20Transfer> {
    let mut result = vec![];

    for msg in msgs.iter().map(|m| &m.msg) {
        match msg {
            CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, funds: _, msg }) => {
                match from_binary(&msg).unwrap() {
                    Cw20ExecuteMsg::Transfer { recipient, amount } => {
                        result.push(Cw20Transfer {
                            contract_addr: contract_addr.to_string(),
                            recipient,
                            amount,
                        });
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    result
}