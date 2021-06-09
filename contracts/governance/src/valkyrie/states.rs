use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


const VALKYRIE_CONFIG: Item<ValkyrieConfig> = Item::new("valkyrie-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValkyrieConfig {
    pub campaign_code_whitelist: Vec<u64>,
    pub boost_contract: Option<Addr>,
}

impl ValkyrieConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        VALKYRIE_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ValkyrieConfig> {
        VALKYRIE_CONFIG.load(storage)
    }
}


const CAMPAIGN_CODES: Map<&[u8], CampaignCode> = Map::new("campaign-code");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignCode {
    pub code_id: u64,
    pub source_code_url: String,
    pub description: String,
    pub maintainer: Option<String>,
}

impl CampaignCode {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_CODES.save(storage, &self.code_id.to_be_bytes(), self)
    }

    pub fn load(storage: &dyn Storage, code_id: &u64) -> StdResult<CampaignCode> {
        CAMPAIGN_CODES.load(storage, &code_id.to_be_bytes())
    }
}