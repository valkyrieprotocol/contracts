use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use valkyrie::{
    common::{ContractResult, OrderBy},
    distributor::query_msgs::{DistributorInfoResponse, DistributorInfosResponse},
    errors::ContractError,
};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("config");
const DISTRIBUTOR: Map<&Addr, Uint128> = Map::new("distributor");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub governance: Addr,
    pub token_contract: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_governance(&self, address: &Addr) -> bool {
        self.governance.eq(address)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributorInfo {
    pub distributor: Addr,
    pub spend_limit: Uint128,
}

impl DistributorInfo {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        DISTRIBUTOR.save(storage, &self.distributor, &self.spend_limit)
    }

    pub fn remove(&self, storage: &mut dyn Storage) {
        DISTRIBUTOR.remove(storage, &self.distributor)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> ContractResult<DistributorInfo> {
        Ok(DistributorInfo {
            distributor: address.clone(),
            spend_limit: DISTRIBUTOR
                .load(storage, address)
                .map_err(|_| ContractError::NotFound {})?,
        })
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<DistributorInfosResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(Bound::exclusive);
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, Order::Ascending),
            _ => (None, start_after, Order::Descending),
        };

        let distributors: StdResult<Vec<DistributorInfoResponse>> = DISTRIBUTOR
            .range(storage, min, max, order_by)
            .take(limit)
            .map(|item| {
                let (k, v) = item?;
                Ok(DistributorInfoResponse {
                    distributor: String::from_utf8(k)?,
                    spend_limit: v,
                })
            })
            .collect();

        Ok(DistributorInfosResponse {
            distributors: distributors?,
        })
    }
}
