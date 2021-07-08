use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw20::Denom;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::campaign_manager::query_msgs::{CampaignResponse, CampaignsResponse};
use valkyrie::common::OrderBy;
use valkyrie::pagination::addr_range_option;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub governance: Addr,
    pub fund_manager: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_governance(&self, address: &Addr) -> bool {
        self.governance == *address
    }
}

pub fn is_governance(storage: &dyn Storage, address: &Addr) -> bool {
    ContractConfig::load(storage).unwrap().is_governance(address)
}

const CAMPAIGN_CONFIG: Item<CampaignConfig> = Item::new("campaign-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfig {
    pub creation_fee_token: Addr,
    pub creation_fee_amount: Uint128,
    pub creation_fee_recipient: Addr,
    pub code_id: u64,
    pub distribution_denom_whitelist: Vec<Denom>,
    pub withdraw_fee_rate: Decimal,
    pub withdraw_fee_recipient: Addr,
    pub deactivate_period: u64,
}

impl CampaignConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CampaignConfig> {
        CAMPAIGN_CONFIG.load(storage)
    }
}

const BOOSTER_CONFIG: Item<BoosterConfig> = Item::new("booster-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BoosterConfig {
    pub booster_token: Addr,
    pub drop_ratio: Decimal,
    pub activity_ratio: Decimal,
    pub plus_ratio: Decimal,
    pub activity_multiplier: Decimal,
    pub min_participation_count: u64,
}

impl BoosterConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        BOOSTER_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<BoosterConfig> {
        BOOSTER_CONFIG.load(storage)
    }
}

const CREATE_CAMPAIGN_CONTEXT: Item<CreateCampaignContext> = Item::new("create-campaign-context");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateCampaignContext {
    pub code_id: u64,
    pub creator: Addr,
}

impl CreateCampaignContext {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CREATE_CAMPAIGN_CONTEXT.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CreateCampaignContext> {
        CREATE_CAMPAIGN_CONTEXT.load(storage)
    }

    #[cfg(test)]
    pub fn may_load(storage: &dyn Storage) -> StdResult<Option<CreateCampaignContext>> {
        CREATE_CAMPAIGN_CONTEXT.may_load(storage)
    }

    pub fn clear(storage: &mut dyn Storage) {
        CREATE_CAMPAIGN_CONTEXT.remove(storage)
    }
}


const CAMPAIGN: Map<&Addr, Campaign> = Map::new("campaign");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Campaign {
    pub code_id: u64,
    pub address: Addr,
    pub creator: Addr,
    pub created_height: u64,
}

impl Campaign {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN.save(storage, &self.address, self)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<Campaign> {
        CAMPAIGN.load(storage, address)
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<CampaignsResponse> {
        let range_option = addr_range_option(start_after, limit, order_by);

        let campaigns = CAMPAIGN
            .range(storage, range_option.min, range_option.max, range_option.order_by)
            .take(range_option.limit)
            .map(|item| {
                let (_, campaign) = item?;

                Ok(CampaignResponse {
                    code_id: campaign.code_id,
                    address: campaign.address.to_string(),
                    creator: campaign.creator.to_string(),
                    created_height: campaign.created_height,
                })
            })
            .collect::<StdResult<Vec<CampaignResponse>>>()?;

        Ok(CampaignsResponse {
            campaigns,
        })
    }
}