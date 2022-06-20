use cosmwasm_std::{
    Addr, BankMsg, Binary, Coin, CosmosMsg, StdResult, to_binary, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use serde::Serialize;

pub fn native_send(
    denom: String,
    recipient: &Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    //only luna
    Ok(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            amount,
            denom,
        }],
    }))
}

pub fn cw20_transfer(token: &Addr, recipient: &Addr, amount: Uint128) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        funds: vec![],
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
        funds: vec![],
        label: "campaign".to_string(),
    })
}

pub fn wasm_execute<T>(contract: &Addr, msg: &T) -> CosmosMsg
where
    T: Serialize + ?Sized {
    wasm_execute_bin(contract, to_binary(&msg).unwrap())
}

pub fn wasm_execute_with_funds<T>(contract: &Addr, funds: Vec<Coin>, msg: &T) -> CosmosMsg
where
    T: Serialize + ?Sized {
    wasm_execute_bin_with_funds(contract, funds, to_binary(msg).unwrap())
}

pub fn wasm_execute_bin(contract: &Addr, msg: Binary) -> CosmosMsg {
    wasm_execute_bin_with_funds(contract, vec![], msg)
}

pub fn wasm_execute_bin_with_funds(contract: &Addr, funds: Vec<Coin>, msg: Binary) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.to_string(),
        funds,
        msg,
    })
}
