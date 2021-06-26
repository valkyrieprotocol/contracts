use cosmwasm_std::{Addr, Api, QuerierWrapper, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::cw20::query_balance;
use crate::utils::decompress_addr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Denom {
    Native(String),
    Token(String),
}

impl Denom {
    pub fn to_cw20(&self, api: &dyn Api) -> cw20::Denom {
        match self {
            Denom::Native(denom) => cw20::Denom::Native(denom.to_string()),
            Denom::Token(contract_addr) => {
                cw20::Denom::Cw20(api.addr_validate(contract_addr).unwrap())
            }
        }
    }

    pub fn from_cw20(denom: cw20::Denom) -> Self {
        match denom {
            cw20::Denom::Native(denom) => Denom::Native(denom),
            cw20::Denom::Cw20(contract_addr) => Denom::Token(contract_addr.to_string()),
        }
    }

    pub fn load_balance(
        &self,
        querier: &QuerierWrapper,
        api: &dyn Api,
        address: Addr,
    ) -> StdResult<u128> {
        query_balance(querier, api, self.to_cw20(api), address)
    }
}

impl fmt::Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Denom::Native(denom) => write!(f, "{}", denom),
            Denom::Token(addr) => write!(f, "{}", addr),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Referrer {
    Address(String),
    Compressed(String),
}

impl Referrer {
    pub fn to_address(&self, api: &dyn Api) -> StdResult<Addr> {
        match self {
            Referrer::Address(v) => api.addr_validate(v),
            Referrer::Compressed(v) => api.addr_validate(&decompress_addr(v)),
        }
    }
}
