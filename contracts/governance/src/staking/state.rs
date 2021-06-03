use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, bucket_read, ReadonlyBucket, ReadonlySingleton, Singleton, singleton, singleton_read};

use valkyrie::governance::models::VoterInfo;

static KEY_STAKING_CONFIG: &[u8] = b"staking-config";
static KEY_STAKING_STATE: &[u8] = b"staking-state";
static PREFIX_STAKER_STATE: &[u8] = b"staker-state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingConfig {}

impl StakingConfig {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<StakingConfig> {
        singleton(storage, KEY_CONFIG)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<StakingConfig> {
        singleton_read(storage, KEY_CONFIG)
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingState {
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

impl StakingState {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<StakingState> {
        singleton(storage, KEY_STAKING_STATE)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<StakingState> {
        singleton_read(storage, KEY_STAKING_STATE)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        StakingState::singleton(storage).save(self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<StakingState> {
        StakingState::singleton_read(storage).load()
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerState {
    pub address: CanonicalAddr,
    pub share: Uint128,
    // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

impl StakerState {
    pub fn default(address: &CanonicalAddr) -> StakerState {
        StakerState {
            address: CanonicalAddr::from(address),
            share: Uint128::zero(),
            locked_balance: vec![],
        }
    }

    pub fn bucket(storage: &mut dyn Storage) -> Bucket<StakerState> {
        bucket(storage, PREFIX_STAKER_STATE)
    }

    pub fn bucket_read(storage: &dyn Storage) -> ReadonlyBucket<StakerState> {
        bucket_read(storage, PREFIX_STAKER_STATE)
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        StakerState::bucket(storage).save(self.address.as_slice(), self)
    }

    pub fn load(storage: &dyn Storage, address: &CanonicalAddr) -> StdResult<StakerState> {
        StakerState::bucket_read(storage).load(address.as_slice())
    }

    pub fn may_load(storage: &dyn Storage, address: &CanonicalAddr) -> StdResult<Option<StakerState>> {
        StakerState::bucket_read(storage).may_load(StakerState::key_of(address))
    }
}