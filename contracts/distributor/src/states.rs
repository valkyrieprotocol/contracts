use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use valkyrie::{
    common::{ContractResult, OrderBy},
    distributor::execute_msgs::BoosterConfig,
    distributor::query_msgs::{CampaignInfoResponse, CampaignInfosResponse},
    errors::ContractError,
};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract_config");
const CAMPAIGN: Map<&Addr, Uint128> = Map::new("campaign");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub governance: Addr,
    pub token_contract: Addr,
    pub terraswap_router: Addr,
    pub booster_config: BoosterConfig,
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
pub struct CampaignInfo {
    pub campaign: Addr,
    pub spend_limit: Uint128,
}

impl CampaignInfo {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN.save(storage, &self.campaign, &self.spend_limit)
    }

    pub fn remove(&self, storage: &mut dyn Storage) {
        CAMPAIGN.remove(storage, &self.campaign)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> ContractResult<CampaignInfo> {
        Ok(CampaignInfo {
            campaign: address.clone(),
            spend_limit: CAMPAIGN
                .load(storage, address)
                .map_err(|_| ContractError::NotFound {})?,
        })
    }

    #[cfg(test)]
    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<CampaignInfo>> {
        Ok(CAMPAIGN.may_load(storage, address)?.map(|spend_limit| {
            CampaignInfo {
                campaign: address.clone(),
                spend_limit,
            }
        }))
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<CampaignInfosResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(Bound::exclusive);
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, Order::Ascending),
            _ => (None, start_after, Order::Descending),
        };

        let campaigns: StdResult<Vec<CampaignInfoResponse>> = CAMPAIGN
            .range(storage, min, max, order_by)
            .take(limit)
            .map(|item| {
                let (k, v) = item?;
                Ok(CampaignInfoResponse {
                    campaign_addr: String::from_utf8(k)?,
                    spend_limit: v,
                })
            })
            .collect();

        Ok(CampaignInfosResponse {
            campaigns: campaigns?,
        })
    }
}
