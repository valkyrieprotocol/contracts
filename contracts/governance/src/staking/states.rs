use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::governance::enumerations::PollStatus;
use valkyrie::governance::models::VoteInfo;

use crate::poll::states::Poll;

// static KEY_STAKING_CONFIG: &[u8] = b"staking-config";

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct StakingConfig {}
//
// impl StakingConfig {
//     pub fn singleton(storage: &mut dyn Storage) -> Singleton<StakingConfig> {
//         singleton(storage, KEY_STAKING_CONFIG)
//     }
//
//     pub fn singleton_read(storage: &dyn Storage) -> ReadonlySingleton<StakingConfig> {
//         singleton_read(storage, KEY_STAKING_CONFIG)
//     }
// }


const STAKING_STATE: Item<StakingState> = Item::new("staking-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingState {
    pub total_share: Uint128,
}

impl StakingState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKING_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<StakingState> {
        STAKING_STATE.load(storage)
    }
}


const STAKER_STATES: Map<&Addr, StakerState> = Map::new("staker-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerState {
    pub address: Addr,
    pub share: Uint128,
    // total staked balance
    pub votes: Vec<(u64, VoteInfo)>, // maps poll_id to weight voted
}

impl StakerState {
    pub fn default(address: &Addr) -> StakerState {
        StakerState {
            address: address.clone(),
            share: Uint128::zero(),
            votes: vec![],
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STAKER_STATES.save(storage, &self.address, self)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<StakerState> {
        STAKER_STATES.load(storage, address)
    }

    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<StakerState>> {
        STAKER_STATES.may_load(storage, address)
    }

    pub fn load_safe(storage: &dyn Storage, address: &Addr) -> StdResult<StakerState> {
        Ok(STAKER_STATES.may_load(storage, address)?.unwrap_or(StakerState::default(address)))
    }

    pub fn clean_votes(&mut self, storage: &dyn Storage) -> () {
        self.votes.retain(|(poll_id, _)| {
            let poll = Poll::load(storage, &poll_id).unwrap();

            poll.status == PollStatus::InProgress
        });
    }

    // quorum 을 넘기지 못해 total_deposit 만 차감되어 contract balance 가 실제 staking 수량보다 많을 수 있다.
    // 이를 스테이커들에게 분배하기 위해 스테이커들의 실제 잔고를 계산할 때는
    // 전체 스테이킹 수량 중 자신이 스테이킹한 수량이 어느정도의 비중을 차지하는지를 따져서
    // 해당 비중만큼을 스테이커의 잔고로 취급한다.
    // TODO: 근데 이러면 언스테이킹 했다가 다시 스테이킹을 해야 실제로 이득이 되는 것 아닌가?
    // TODO: 실제로 지급한것이 아니고 contract balance - staking amount 만큼을 그냥 스테이커들이 비중만큼 공유하고 있기 때문에
    // TODO: unstaking 하고 다시 staking 해야 실제적인 이득으로 돌아올 것 같은데.. 확인이 필요하다.
    pub fn load_balance(&self, storage: &dyn Storage, contract_available_balance: Uint128) -> StdResult<Uint128> {
        let staking_state = StakingState::load(storage)?;

        if staking_state.total_share.is_zero() {
            return Ok(Uint128::zero())
        }

        let staker_balance = self.share.multiply_ratio(
            contract_available_balance,
            staking_state.total_share,
        );

        return Ok(staker_balance);
    }

    // removes not in-progress poll voter info & unlock tokens
    // and returns the largest locked amount in participated polls.
    pub fn get_locked_balance(&self) -> Uint128 {
        self.votes.iter()
            .map(|(_, v)| v.amount)
            .max()
            .unwrap_or_default()
    }

    pub fn can_vote(&self, storage: &dyn Storage, contract_available_balance: Uint128, amount: Uint128) -> StdResult<bool> {
        let balance = self.load_balance(storage, contract_available_balance)?;

        Ok(balance >= amount)
    }

    pub fn vote(&mut self, poll_id: u64, vote: VoteInfo) {
        self.votes.push((poll_id, vote));
    }
}