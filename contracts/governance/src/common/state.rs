use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton, singleton, singleton_read};

static KEY_CONTRACT_CONFIG: &[u8] = b"contract-config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: CanonicalAddr,
    pub token_contract: CanonicalAddr,
    pub boost_contract: Option<CanonicalAddr>,
}

impl ContractConfig {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<ContractConfig> {
        singleton(storage, KEY_CONTRACT_CONFIG)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<ContractConfig> {
        singleton_read(storage, KEY_CONTRACT_CONFIG)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        ContractConfig::singleton(storage).save(self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        ContractConfig::singleton_read(storage).load()
    }

    pub fn is_admin(&self, address: CanonicalAddr) -> bool {
        self.admin == address
    }

    pub fn is_token_contract(&self, address: CanonicalAddr) -> bool {
        self.token_contract == address
    }
}