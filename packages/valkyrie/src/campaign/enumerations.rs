use cosmwasm_std::{Addr, Api, QuerierWrapper, StdResult};

use crate::cw20::{query_cw20_balance, query_balance};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Denom {
    Native(String),
    Token(String),
}

impl Denom {
    pub fn to_cw20(&self, api: &dyn Api) -> cw20::Denom {
        match self {
            Denom::Native(denom) => cw20::Denom(denom),
            Denom::Cw20(contract_addr) => cw20::Denom(api.addr_validate(contract_addr).unwrap()),
        }
    }

    pub fn load_balance(&self, querier: &QuerierWrapper, api: &dyn Api, address: Addr) -> StdResult<u128> {
        query_balance(querier, api, self.to_cw20(api), address)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Referrer {
    Address(String),
    Compressed(String),
}