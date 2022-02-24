use std::cmp::max;
use cosmwasm_std::{Addr, BlockInfo, QuerierWrapper, StdError, StdResult, Storage, Timestamp, Uint128, Decimal};
use cw20::Denom;
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::common::OrderBy;
use valkyrie::campaign_manager::query_msgs::ReferralRewardLimitOptionResponse;
use valkyrie::campaign::query_msgs::ReferralRewardLimitAmount;
use valkyrie::governance::query_msgs::StakerStateResponse;

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;


const CAMPAIGN_CONFIG: Item<CampaignConfig> = Item::new("campaign_config");
const ADMIN_NOMINEE: Item<Addr> = Item::new("admin_nominee");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignConfig {
    pub governance: Addr,
    pub campaign_manager: Addr,
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub deposit_denom: Option<Denom>,
    pub deposit_amount: Uint128,
    pub deposit_lock_period: u64,
    pub vp_token: Addr,
    pub vp_burn_amount: Uint128,
    pub qualifier: Option<Addr>,
    pub qualification_description: Option<String>,
    pub admin: Addr,
    pub creator: Addr,
    pub created_at: Timestamp,
}

impl CampaignConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CampaignConfig> {
        CAMPAIGN_CONFIG.load(storage)
    }

    pub fn may_load_admin_nominee(storage: &dyn Storage) -> StdResult<Option<Addr>> {
        ADMIN_NOMINEE.may_load(storage)
    }

    pub fn save_admin_nominee(storage: &mut dyn Storage, address: &Addr) -> StdResult<()> {
        ADMIN_NOMINEE.save(storage, address)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admin == *address
    }

    pub fn require_deposit(&self) -> bool {
        self.deposit_denom.is_some()
    }
}

pub fn is_admin(storage: &dyn Storage, address: &Addr) -> StdResult<bool> {
    CampaignConfig::load(storage).map(|c| c.is_admin(address))
}


const CAMPAIGN_STATE: Item<CampaignState> = Item::new("campaign_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignState {
    pub actor_count: u64,
    pub participation_count: u64,
    pub cumulative_participation_reward_amount: Uint128,
    pub cumulative_referral_reward_amount: Uint128,
    pub balances: Vec<(Denom, Uint128)>,
    pub locked_balances: Vec<(Denom, Uint128)>,
    pub deposit_amount: Uint128,
    pub active_flag: bool,
    pub last_active_height: Option<u64>,
    pub chain_id: String,
}

impl CampaignState {
    pub fn new(chain_id: String) -> CampaignState {
        CampaignState {
            actor_count: 0,
            participation_count: 0,
            cumulative_participation_reward_amount: Uint128::zero(),
            cumulative_referral_reward_amount: Uint128::zero(),
            balances: vec![],
            locked_balances: vec![],
            deposit_amount: Uint128::zero(),
            active_flag: false,
            last_active_height: None,
            chain_id,
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CampaignState> {
        CAMPAIGN_STATE.load(storage)
    }

    pub fn is_active(
        &self,
        campaign_config: &CampaignConfig,
        querier: &QuerierWrapper,
        block: &BlockInfo,
    ) -> StdResult<bool> {
        if !self.active_flag {
            return Ok(false);
        }

        if self.chain_id != block.chain_id {
            return Ok(false);
        }

        let global_campaign_config = load_global_campaign_config(
            querier,
            &campaign_config.campaign_manager,
        )?;

        Ok(global_campaign_config.deactivate_period + self.last_active_height.unwrap_or_default() >= block.height)
    }

    pub fn is_pending(&self) -> bool {
        self.last_active_height.is_none()
    }

    pub fn balance(&self, denom: &Denom) -> Balance {
        for (denomination, balance) in self.balances.iter() {
            if *denomination == *denom {
                let locked_balance = self.locked_balance(denom);

                return Balance { total: *balance, locked: locked_balance };
            }
        }

        Balance::default()
    }

    pub fn deposit(&mut self, denom: &Denom, amount: &Uint128) {
        match self.balances.iter_mut().find(|e| e.0 == *denom) {
            Some(balance) => balance.1 += amount,
            None => self.balances.push((denom.clone(), amount.clone())),
        }
    }

    pub fn withdraw(&mut self, denom: &Denom, amount: &Uint128) -> StdResult<Uint128> {
        match self.balances.iter_mut().find(|e| e.0 == *denom) {
            Some(balance) => {
                balance.1 = balance.1.checked_sub(*amount)?;
                Ok(balance.1)
            }
            None => Err(StdError::overflow(Uint128::zero().checked_sub(*amount).unwrap_err())),
        }
    }

    pub fn validate_balance(&self) -> StdResult<()> {
        for (denom, locked_balance) in self.locked_balances.iter() {
            let balance = self.total_balance(denom);

            if balance < *locked_balance {
                return Err(StdError::generic_err("locked balance can't greater than balance"));
            }
        }

        Ok(())
    }

    pub fn locked_balance(&self, denom: &Denom) -> Uint128 {
        for (locked_denom, locked_amount) in self.locked_balances.iter() {
            if *locked_denom == *denom {
                return locked_amount.clone();
            }
        }

        Uint128::zero()
    }

    fn total_balance(&self, denom: &Denom) -> Uint128 {
        for (balance_denom, balance_amount) in self.balances.iter() {
            if *balance_denom == *denom {
                return balance_amount.clone();
            }
        }

        Uint128::zero()
    }

    pub fn lock_balance(&mut self, denom: &Denom, amount: &Uint128) {
        match self.locked_balances.iter_mut().find(|e| e.0 == *denom) {
            Some(locked_balance) => locked_balance.1 += amount,
            None => self.locked_balances.push((denom.clone(), amount.clone())),
        }
    }

    pub fn unlock_balance(&mut self, denom: &Denom, amount: &Uint128) -> StdResult<Uint128> {
        match self.locked_balances.iter_mut().find(|e| e.0 == *denom) {
            Some(locked_balance) => {
                locked_balance.1 = locked_balance.1.checked_sub(*amount)?;
                Ok(locked_balance.1)
            }
            None => Err(StdError::overflow(Uint128::zero().checked_sub(*amount).unwrap_err())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Balance {
    pub total: Uint128,
    pub locked: Uint128,
}

impl Balance {
    #[allow(dead_code)]
    pub fn available(&self) -> Uint128 {
        self.total.checked_sub(self.locked).unwrap()
    }
}

pub fn is_pending(storage: &dyn Storage) -> StdResult<bool> {
    Ok(CampaignState::load(storage)?.is_pending())
}

const REWARD_CONFIG: Item<RewardConfig> = Item::new("reward_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardConfig {
    pub participation_reward_denom: Denom,
    pub participation_reward_amount: Uint128,
    pub participation_reward_lock_period: u64,
    pub participation_reward_distribution_schedule: Vec<(u64, u64, Decimal)>,
    pub referral_reward_token: Addr,
    pub referral_reward_amounts: Vec<Uint128>,
    pub referral_reward_lock_period: u64,
}

impl RewardConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        self.validate()?;
        REWARD_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<RewardConfig> {
        REWARD_CONFIG.load(storage)
    }

    pub fn validate(&self) -> StdResult<()> {
        if self.referral_reward_amounts.is_empty()
            || self.referral_reward_amounts.iter().all(|v| v.is_zero()) {
            return Err(StdError::generic_err("Invalid reward scheme"));
        }

        Ok(())
    }
}


const ACTORS: Map<&Addr, Actor> = Map::new("actor");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Actor {
    pub address: Addr,
    pub referrer: Option<Addr>,
    pub participation_reward_amounts: Vec<(Uint128, u64)>,
    pub referral_reward_amounts: Vec<(Uint128, u64)>,
    pub cumulative_participation_reward_amount: Uint128,
    pub cumulative_referral_reward_amount: Uint128,
    pub participation_count: u64,
    pub referral_count: u64,
    pub last_participated_at: Timestamp,
}

impl Actor {
    pub fn new(address: Addr, referrer: Option<Addr>) -> Actor {
        Actor {
            address,
            referrer,
            participation_reward_amounts: vec![],
            referral_reward_amounts: vec![],
            cumulative_participation_reward_amount: Uint128::zero(),
            cumulative_referral_reward_amount: Uint128::zero(),
            participation_count: 0,
            referral_count: 0,
            last_participated_at: Timestamp::default(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        ACTORS.save(storage, &self.address, self)
    }

    #[allow(dead_code)]
    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<Actor> {
        ACTORS.load(storage, address)
    }

    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<Actor>> {
        ACTORS.may_load(storage, address)
    }

    pub fn load_referrers(&self, storage: &dyn Storage, distance_limit: usize) -> StdResult<Vec<Actor>> {
        let mut result = vec![];

        let mut referrer = self.referrer.clone();
        for _ in 0..distance_limit {
            if referrer.is_none() {
                break;
            }

            let referrer_participation = Self::may_load(storage, referrer.as_ref().unwrap())?;
            if referrer_participation.is_none() {
                break;
            }
            let referrer_participation = referrer_participation.unwrap();
            referrer = referrer_participation.referrer.clone();
            result.push(referrer_participation)
        }

        Ok(result)
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<Addr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Actor>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(|v| Bound::exclusive(v.as_str().as_bytes()));
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
            _ => (None, start_after, OrderBy::Desc),
        };

        ACTORS
            .range(storage, min, max, order_by.into())
            .take(limit)
            .map(|item| {
                let (_, v) = item?;
                Ok(v)
            })
            .collect()
    }

    pub fn add_participation_reward(&mut self, amount: Uint128, unlock_height: u64) {
        self.participation_reward_amounts.push((amount, unlock_height));
    }

    pub fn add_referral_reward(&mut self, amount: Uint128, unlock_height: u64) {
        self.referral_reward_amounts.push((amount, unlock_height));
    }

    pub fn participation_reward_amount(&self, storage:&dyn Storage, height: u64) -> StdResult<(Uint128, Uint128)> {
        let mut unlocked_amount = Uint128::zero();
        let mut total_amount = Uint128::zero();

        let reward_config = RewardConfig::load(storage)?;
        let actor_state = ActorState::load_or_default(storage, &self.address)?;

        let mut last_end_height = 0;

        for (_ , unlock_height) in self.participation_reward_amounts.iter() {
            let participate_height = unlock_height - reward_config.participation_reward_lock_period;
            for (_, end, _) in reward_config.participation_reward_distribution_schedule.iter() {
                last_end_height = end.clone() + participate_height.clone();
            }
        }



        for (amount, unlock_height) in self.participation_reward_amounts.iter() {
            total_amount += amount;

            let participate_height = unlock_height - reward_config.participation_reward_lock_period;

            for (start, end, distribution_ratio) in reward_config.participation_reward_distribution_schedule.iter() {
                //s.0 = begin block height of this schedule
                //s.1 = (Optional) end block height of this schedule

                let start_height = start.clone() + participate_height.clone();
                let end_height = end.clone() + participate_height.clone();

                let distribution_amount_schedule = amount.clone() * distribution_ratio.clone();

                if start_height <= height && end_height >= actor_state.participation_reward_last_distributed {
                    if start_height == end_height {
                        unlocked_amount += distribution_amount_schedule;
                    } else {
                        // min(s.1, block_height) - max(s.0, last_distributed)
                        let passed_blocks =
                            std::cmp::min(end_height, height) - std::cmp::max(start_height, actor_state.participation_reward_last_distributed);

                        let num_blocks = end_height - start_height;
                        let distribution_amount_per_block: Decimal = Decimal::from_ratio(distribution_amount_schedule, num_blocks);
                        // distribution_amount_per_block = distribution amount of this schedule / blocks count of this schedule.
                        unlocked_amount +=
                            distribution_amount_per_block * Uint128::new(passed_blocks as u128);
                    }
                }

                last_end_height = max(end_height, last_end_height) ;
            }
            // self.last_distributed = block_height; >>> will be executed in 'claim_participation_reward'
        }

        if last_end_height <= height {
            //remaining reward can be made calculation(like 1uusd).
            //so, after all schedule, user can claim all remainings.
            let actor_state = ActorState::load_or_default(storage, &self.address)?;
            let unlocked_amount = total_amount - actor_state.claimed_participation_reward_amount;
            Ok((unlocked_amount, Uint128::zero()))
        } else {
            Ok((unlocked_amount, total_amount - unlocked_amount))
        }
    }

    pub fn referral_reward_amount(&self, height: u64) -> (Uint128, Uint128) {
        let mut unlocked_amount = Uint128::zero();
        let mut locked_amount = Uint128::zero();

        for (amount, unlock_height) in self.referral_reward_amounts.iter() {
            if *unlock_height <= height {
                unlocked_amount += *amount;
            } else {
                locked_amount += *amount;
            }
        }

        (unlocked_amount, locked_amount)
    }

    pub fn claim_referral_reward_amount(&mut self, height: u64) -> Uint128 {
        let mut unlocked_amount = Uint128::zero();
        let mut locked_amounts: Vec<(Uint128, u64)> = vec![];

        for (amount, unlock_height) in self.referral_reward_amounts.iter() {
            if *unlock_height <= height {
                unlocked_amount += *amount;
            } else {
                locked_amounts.push((*amount, *unlock_height))
            }
        }

        self.referral_reward_amounts = locked_amounts;

        unlocked_amount
    }
}

const ACTORSTATE: Map<&Addr, ActorState> = Map::new("actor-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ActorState {
    pub address:Addr,

    pub participation_reward_last_distributed: u64,
    pub claimed_participation_reward_amount: Uint128,

    pub referral_reward_last_distributed: u64,
    pub claimed_referral_reward_amount: Uint128,
}

impl ActorState {
    pub fn new(address: Addr) -> ActorState {
        ActorState {
            address,
            participation_reward_last_distributed: 0,
            claimed_participation_reward_amount: Uint128::zero(),
            referral_reward_last_distributed: 0,
            claimed_referral_reward_amount: Uint128::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        ACTORSTATE.save(storage, &self.address, self)
    }

    #[allow(dead_code)]
    pub fn load_or_default(storage: &dyn Storage, address: &Addr) -> StdResult<ActorState> {
        let loaded = ACTORSTATE.may_load(storage, address)?;
        if let Some(loaded) = loaded {
            Ok(loaded)
        } else {
            Ok(ActorState::new(address.clone()))
        }
    }
}


const DEPOSITS: Map<&Addr, Deposit> = Map::new("deposit");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Deposit {
    pub owner: Addr,
    pub deposit_amount: Uint128,
    pub locked_amounts: Vec<(Uint128, u64)>,
}

impl Deposit {
    pub fn new(owner: Addr) -> Deposit {
        Deposit {
            owner,
            deposit_amount: Uint128::zero(),
            locked_amounts: vec![],
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        DEPOSITS.save(storage, &self.owner, self)
    }

    pub fn load(storage: &dyn Storage, owner: &Addr) -> StdResult<Deposit> {
        DEPOSITS.load(storage, owner)
    }

    pub fn load_or_new(storage: &dyn Storage, owner: &Addr) -> StdResult<Deposit> {
        Ok(DEPOSITS.may_load(storage, owner)
            ?.unwrap_or_else(|| Self::new(owner.clone())))
    }

    pub fn clear(&mut self, height: u64) {
        let mut locked_amounts = vec![];

        loop {
            match self.locked_amounts.pop() {
                Some((locked_amount, unlock_height)) => {
                    if unlock_height > height {
                        locked_amounts.push((locked_amount, unlock_height));
                    }
                },
                None => break,
            }
        }

        self.locked_amounts = locked_amounts;
    }

    pub fn locked_amount(&self, height: u64) -> Uint128 {
        self.locked_amounts.iter()
            .fold(Uint128::zero(), |locked_amount, (amount, unlock_height)| {
                if *unlock_height > height {
                    locked_amount + *amount
                } else {
                    locked_amount
                }
            })
    }

    pub fn balance(&self, height: u64) -> StdResult<Uint128> {
        Ok(self.deposit_amount.checked_sub(self.locked_amount(height))?)
    }

    pub fn lock(&mut self, amount: Uint128, height: u64, lock_period: u64) -> StdResult<()> {
        self.balance(height)?.checked_sub(amount)?; //check overflow

        self.locked_amounts.push((amount, height + lock_period));

        Ok(())
    }
}


const QUALIFY_PARTICIPATION_CONTEXT: Item<QualifyParticipationContext> = Item::new("qualify_participation_context");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualifyParticipationContext {
    pub actor: Addr,
    pub referrer: Option<Addr>,
}

impl QualifyParticipationContext {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        QUALIFY_PARTICIPATION_CONTEXT.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<QualifyParticipationContext> {
        QUALIFY_PARTICIPATION_CONTEXT.load(storage)
    }

    pub fn clear(storage: &mut dyn Storage) {
        QUALIFY_PARTICIPATION_CONTEXT.remove(storage)
    }
}

pub fn load_global_campaign_config(
    querier: &QuerierWrapper,
    campaign_manager: &Addr,
) -> StdResult<valkyrie::campaign_manager::query_msgs::ConfigResponse> {
    querier.query_wasm_smart(
        campaign_manager,
        &valkyrie::campaign_manager::query_msgs::QueryMsg::Config {},
    )
}

pub fn calc_referral_reward_limit(
    limit_option: &ReferralRewardLimitOptionResponse,
    campaign_config: &CampaignConfig,
    reward_config: &RewardConfig,
    querier: &QuerierWrapper,
    address: &Addr,
) -> StdResult<ReferralRewardLimitAmount> {
    let base_limit_amount = reward_config.referral_reward_amounts.iter().sum::<Uint128>()
        .checked_mul(Uint128::from(limit_option.base_count))?;

    let gov_staker_state: StakerStateResponse = querier.query_wasm_smart(
        &campaign_config.governance,
        &valkyrie::governance::query_msgs::QueryMsg::StakerState {
            address: address.to_string(),
        },
    )?;
    let gov_staking_amount = gov_staker_state.balance;

    let actor_limit_amount = gov_staking_amount * Decimal::percent(limit_option.percent_for_governance_staking as u64);

    let limit_amount = base_limit_amount + actor_limit_amount;

    Ok(ReferralRewardLimitAmount {
        address: address.to_string(),
        limit_amount,
        base_limit_amount,
        actor_limit_amount,
    })
}
