use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{Bucket, bucket, bucket_read, ReadonlyBucket, ReadonlySingleton, Singleton, singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_VALKYRIE_CONFIG: &[u8] = b"valkyrie-config";
static PREFIX_CAMPAIGN_CODE: &[u8] = b"campaign-code";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValkyrieConfig {
    pub campaign_code_whitelist: Vec<u64>,
    pub boost_contract: Option<CanonicalAddr>,
}

impl ValkyrieConfig {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<ValkyrieConfig> {
        singleton(storage, KEY_VALKYRIE_CONFIG)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<ValkyrieConfig> {
        singleton_read(storage, KEY_VALKYRIE_CONFIG)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        ValkyrieConfig::singleton(storage).save(self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ValkyrieConfig> {
        ValkyrieConfig::singleton_read(storage).load()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignCode {
    pub code_id: u64,
    pub source_code_url: String,
    pub description: String,
    pub maintainer: Option<String>,
}

impl CampaignCode {
    pub fn bucket(storage: &mut dyn Storage) -> Bucket<CampaignCode> {
        bucket(storage, PREFIX_CAMPAIGN_CODE)
    }

    pub fn bucket_read(storage: &dyn Storage) -> ReadonlyBucket<CampaignCode> {
        bucket_read(storage, PREFIX_CAMPAIGN_CODE)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CampaignCode::bucket(storage).save(&self.code_id.to_be_bytes(), &self)
    }

    pub fn load(storage: &dyn Storage, code_id: &u64) -> StdResult<CampaignCode> {
        CampaignCode::bucket_read(storage).load(&code_id.to_be_bytes())
    }
}