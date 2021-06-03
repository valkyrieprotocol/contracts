use cosmwasm_std::{CanonicalAddr, Storage};
use cosmwasm_storage::{Singleton, ReadonlySingleton, singleton_read};

static KEY_CONTRACT_CONFIG: &[u8] = b"contract-config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: CanonicalAddr,
    pub boost_contract: Option<CanonicalAddr>,
}

impl ContractConfig {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<ContractConfig> {
        singleton(storage, KEY_CONFIG)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<ContractConfig> {
        singleton_read(storage, KEY_CONFIG)
    }
}