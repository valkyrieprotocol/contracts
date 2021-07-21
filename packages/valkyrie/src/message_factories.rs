use crate::terra::extract_tax;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, QuerierWrapper, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

pub fn native_send(
    querier: &QuerierWrapper,
    denom: String,
    recipient: &Addr,
    amount_with_tax: u128,
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            amount: Uint128::from(
                amount_with_tax - extract_tax(querier, denom.to_string(), amount_with_tax)?,
            ),
            denom,
        }],
    }))
}

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

pub fn wasm_instantiate(code_id: u64, admin: Option<Addr>, msg: Binary) -> SubMsg {
    SubMsg::new(WasmMsg::Instantiate {
        admin: admin.map(|v| v.to_string()),
        code_id,
        msg,
        funds: vec![],
        label: String::new(),
    })
}
