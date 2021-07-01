use cosmwasm_std::{Addr, StdResult, Storage, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const FACTORY_CONFIG: Item<FactoryConfig> = Item::new("factory_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FactoryConfig {
    pub governance: Addr,
    pub token_contract: Addr,
    pub distributor: Addr,
    pub burn_contract: Addr,
    pub campaign_code_id: u64,
    pub creation_fee_amount: Uint128,
}

impl FactoryConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        FACTORY_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<FactoryConfig> {
        FACTORY_CONFIG.load(storage)
    }

    pub fn is_governance(&self, address: &Addr) -> bool {
        self.governance.eq(address)
    }

    pub fn is_token_contract(&self, address: &Addr) -> bool {
        self.token_contract.eq(address)
    }
}

pub fn is_governance(storage: &dyn Storage, address: &Addr) -> bool {
    FactoryConfig::load(storage).unwrap().is_governance(address)
}

pub fn is_token_contract(storage: &dyn Storage, address: &Addr) -> bool {
    FactoryConfig::load(storage).unwrap().is_token_contract(address)
}

const CAMPAIGN_CONFIG: Item<CampaignConfig> = Item::new("campaign-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfig {
    pub reward_withdraw_burn_rate: Decimal,
    pub campaign_deactivate_period: u64,
}

impl CampaignConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CampaignConfig> {
        CAMPAIGN_CONFIG.load(storage)
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
    pub created_block: u64,
}

impl Campaign {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN.save(storage, &self.address, self)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<Campaign> {
        CAMPAIGN.load(storage, address)
    }
}