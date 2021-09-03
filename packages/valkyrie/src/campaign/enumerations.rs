use cosmwasm_std::{Addr, Api, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::decompress_addr;

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
