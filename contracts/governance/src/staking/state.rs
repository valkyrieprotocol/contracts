use cosmwasm_std::{CanonicalAddr, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton, singleton_read, Bucket, ReadonlyBucket, bucket_read};
use valkyrie::governance::models::VoterInfo;

static KEY_STAKING_CONFIG: &[u8] = b"staking-config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingConfig {
}

impl StakingConfig {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<StakingConfig> {
        singleton(storage, KEY_CONFIG)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<StakingConfig> {
        singleton_read(storage, KEY_CONFIG)
    }
}


static KEY_STAKING_STATE: &[u8] = b"staking-state";
static PREFIX_STAKING_STATE: &[u8] = b"staking-individual-state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingState {
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingIndividualState {
    pub share: Uint128, // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

impl StakingState {
    pub fn singleton(storage: &mut dyn Storage) -> Singleton<StakingState> {
        singleton(storage, KEY_STAKING_STATE)
    }

    pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<StakingState> {
        singleton_read(storage, KEY_STAKING_STATE)
    }

    pub fn bucket(storage: &mut dyn Storage) -> Bucket<StakingIndividualState> {
        bucket(storage, PREFIX_STAKING_STATE)
    }

    pub fn bucket_read(storage: &mut dyn Storage) -> ReadonlyBucket<StakingIndividualState> {
        bucket_read(storage, PREFIX_STAKING_STATE)
    }
}