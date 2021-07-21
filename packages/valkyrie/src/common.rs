use cosmwasm_std::Order;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::errors::ContractError;

pub type ContractResult<T> = core::result::Result<T, ContractError>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    Asc,
    Desc,
}

impl From<OrderBy> for Order {
    fn from(order_by: OrderBy) -> Self {
        match order_by {
            OrderBy::Asc => Order::Ascending,
            OrderBy::Desc => Order::Descending,
        }
    }
}
