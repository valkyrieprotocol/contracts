use cosmwasm_std::Order;
use cw_storage_plus::Bound;

use crate::common::OrderBy;

pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 10;

pub struct RangeOption {
    pub limit: usize,
    pub min: Option<Bound>,
    pub max: Option<Bound>,
    pub order_by: Order,
}

pub fn addr_range_option(
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> RangeOption {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_after = start_after.map(Bound::exclusive);
    let (min, max, order_by) = match order_by {
        Some(OrderBy::Asc) => (start_after, None, Order::Ascending),
        _ => (None, start_after, Order::Descending),
    };

    RangeOption {
        limit,
        min,
        max,
        order_by,
    }
}
