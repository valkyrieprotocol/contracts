use cosmwasm_std::{Addr, Decimal, QuerierWrapper, StdResult, Storage, Timestamp, Uint128, StdError};
use cw20::Denom;
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::common::{OrderBy, Execution};
use valkyrie::governance::query_msgs::VotingPowerResponse;
use valkyrie::utils::split_uint128;
use valkyrie::campaign::query_msgs::{DropBoosterResponse, ActivityBoosterResponse, PlusBoosterResponse, BoosterResponse};

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: Addr,
    pub governance: Addr,
    pub campaign_manager: Addr,
    pub fund_manager: Addr,
    pub proxies: Vec<Addr>,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admin == *address
    }

    pub fn is_campaign_manager(&self, address: &Addr) -> bool {
        self.campaign_manager == *address
    }

    pub fn can_participate_execution(&self, address: &Addr) -> bool {
        if self.proxies.is_empty() {
            true
        } else {
            self.proxies.contains(address) || self.is_admin(address)
        }
    }
}

pub fn is_admin(storage: &dyn Storage, address: &Addr) -> bool {
    ContractConfig::load(storage).unwrap().is_admin(address)
}

const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInfo {
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub executions: Vec<Execution>,
    pub creator: Addr,
    pub created_at: Timestamp,
    pub created_height: u64,
}

impl CampaignInfo {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_INFO.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CampaignInfo> {
        CAMPAIGN_INFO.load(storage)
    }
}

const CAMPAIGN_STATE: Item<CampaignState> = Item::new("campaign_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignState {
    pub participation_count: u64,
    pub distance_counts: Vec<u64>,
    pub cumulative_distribution_amount: Uint128,
    pub locked_balance: Uint128,
    pub active_flag: bool,
    pub last_active_height: Option<u64>,
}

impl CampaignState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CampaignState> {
        CAMPAIGN_STATE.load(storage)
    }

    pub fn is_active(
        &self,
        storage: &dyn Storage,
        querier: &QuerierWrapper,
        block_height: u64,
    ) -> StdResult<bool> {
        if !self.active_flag {
            return Ok(false);
        }

        let config = ContractConfig::load(storage)?;
        let global_campaign_config = load_global_campaign_config(
            querier,
            &config.campaign_manager,
        )?;

        Ok(global_campaign_config.deactivate_period + self.last_active_height.unwrap_or_default() >= block_height)
    }

    pub fn is_pending(&self) -> bool {
        self.last_active_height.is_none()
    }

    pub fn increase_distance_count(&mut self, distance: u64) {
        match self.distance_counts.get_mut(distance as usize) {
            Some(distance_count) => *distance_count += 1,
            None => self.distance_counts.insert(distance as usize, 1),
        };
    }

    pub fn plus_distribution(&mut self, amount: Uint128) {
        self.cumulative_distribution_amount += amount;
        self.locked_balance += amount;
    }
}

pub fn is_pending(storage: &dyn Storage) -> StdResult<bool> {
    Ok(CampaignState::load(storage)?.is_pending())
}

const DISTRIBUTION_CONFIG: Item<DistributionConfig> = Item::new("distribution_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionConfig {
    pub denom: Denom,
    pub amounts: Vec<Uint128>,
}

impl DistributionConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        self.validate()?;
        DISTRIBUTION_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<DistributionConfig> {
        DISTRIBUTION_CONFIG.load(storage)
    }

    pub fn validate(&self) -> StdResult<()> {
        if self.amounts.is_empty() || self.amounts.iter().all(|v| v.is_zero()) {
            return Err(StdError::generic_err("Invalid reward scheme"));
        }

        Ok(())
    }

    pub fn amounts_sum(&self) -> Uint128 {
        let mut sum = Uint128::zero();
        for amount in self.amounts.iter() {
            sum += amount;
        }
        sum
    }
}


const BOOSTER_STATE: Item<BoosterState> = Item::new("booster_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BoosterState {
    pub recent_booster_id: u64,
}

impl BoosterState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        BOOSTER_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<BoosterState> {
        BOOSTER_STATE.load(storage)
    }
}

pub fn get_booster_id(storage: &mut dyn Storage) -> StdResult<u64> {
    let mut booster_state = BoosterState::load(storage)?;
    booster_state.recent_booster_id += 1;
    booster_state.save(storage)?;

    Ok(booster_state.recent_booster_id)
}

const ACTIVE_BOOSTER: Item<Booster> = Item::new("active_booster_id");
const PREV_BOOSTERS: Map<&[u8], Booster> = Map::new("booster");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Booster {
    pub id: u64,
    pub drop_booster: DropBooster,
    pub activity_booster: ActivityBooster,
    pub plus_booster: PlusBooster,
    pub boosted_at: Timestamp,
    pub finished_at: Option<Timestamp>,
}

impl Booster {
    pub fn load(storage: &dyn Storage, id: u64) -> StdResult<Booster> {
        let active_booster = ACTIVE_BOOSTER.may_load(storage)?;

        if let Some(active_booster) = active_booster {
            if active_booster.id == id {
                return Ok(active_booster);
            }
        }

        PREV_BOOSTERS.load(storage, &id.to_be_bytes())
    }

    pub fn load_active(storage: &dyn Storage) -> StdResult<Booster> {
        ACTIVE_BOOSTER.load(storage)
    }

    pub fn load_prev(storage: &dyn Storage, id: u64) -> StdResult<Booster> {
        PREV_BOOSTERS.load(storage, &id.to_be_bytes())
    }

    pub fn may_load_active(storage: &dyn Storage) -> StdResult<Option<Booster>> {
        ACTIVE_BOOSTER.may_load(storage)
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<BoosterResponse>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(|v| Bound::exclusive(v.to_be_bytes()));
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
            _ => (None, start_after, OrderBy::Desc),
        };

        PREV_BOOSTERS
            .range(storage, min, max, order_by.into())
            .take(limit)
            .map(|item| {
                let (_, v) = item?;
                Ok(v.to_response())
            })
            .collect()
    }

    pub fn is_boosting(storage: &dyn Storage) -> StdResult<bool> {
        Ok(ACTIVE_BOOSTER.may_load(storage)?.is_some())
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        if self.is_active() {
            ACTIVE_BOOSTER.save(storage, self)
        } else {
            PREV_BOOSTERS.save(storage, &self.id.to_be_bytes(), self)
        }
    }

    pub fn finish_with_save(&mut self, storage: &mut dyn Storage, time: Timestamp) -> StdResult<()> {
        self.finished_at = Some(time);

        ACTIVE_BOOSTER.remove(storage);

        self.save(storage)
    }

    pub fn is_active(&self) -> bool {
        self.finished_at.is_none()
    }

    pub fn to_response(&self) -> BoosterResponse {
        BoosterResponse {
            drop_booster: self.drop_booster.to_response(),
            activity_booster: self.activity_booster.to_response(),
            plus_booster: self.plus_booster.to_response(),
            boosted_at: self.boosted_at,
            finished_at: self.finished_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DropBooster {
    pub assigned_amount: Uint128,
    pub calculated_amount: Uint128,
    pub spent_amount: Uint128,
    pub reward_amount: Uint128,
    pub reward_amounts: Vec<Uint128>,
    pub snapped_participation_count: u64,
    pub snapped_distance_counts: Vec<u64>,
}

impl DropBooster {
    pub fn new(
        assigned_amount: Uint128,
        distribution_amounts: Vec<Uint128>,
        participation_count: u64,
        distance_counts: Vec<u64>,
    ) -> DropBooster {
        let reward_amount = if participation_count == 0 {
            Uint128::zero()
        } else {
            assigned_amount
                .checked_div(Uint128::from(participation_count)).unwrap()
        };
        let reward_amounts = split_uint128(
            reward_amount.clone(),
            &distribution_amounts,
        );

        let mut calculated_amount = Uint128::zero();
        for (distance, amount) in reward_amounts.iter().enumerate() {
            let count = distance_counts.get(distance)
                .map_or(Uint128::zero(), |v| Uint128::from(*v));

            calculated_amount += count.checked_mul(*amount).unwrap();
        }

        DropBooster {
            assigned_amount,
            calculated_amount,
            spent_amount: Uint128::zero(),
            reward_amount,
            reward_amounts,
            snapped_participation_count: participation_count,
            snapped_distance_counts: distance_counts,
        }
    }

    pub fn calc_reward_amount(&self, participation: &Participation, booster_id: u64) -> Uint128 {
        let mut result = Uint128::zero();

        for (distance, amount) in self.reward_amounts.iter().enumerate() {
            result += participation.drop_booster_distance_counts(booster_id).get(distance)
                .map_or(Uint128::zero(), |(_, count)| Uint128::from(*count))
                .checked_mul(*amount).unwrap();
        }

        result
    }

    pub fn to_response(&self) -> DropBoosterResponse {
        DropBoosterResponse {
            assigned_amount: self.assigned_amount,
            calculated_amount: self.calculated_amount,
            spent_amount: self.spent_amount,
            reward_amounts: self.reward_amounts.clone(),
            snapped_participation_count: self.snapped_participation_count,
            snapped_distance_counts: self.snapped_distance_counts.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ActivityBooster {
    pub assigned_amount: Uint128,
    pub distributed_amount: Uint128,
    pub reward_amount: Uint128,
    pub reward_amounts: Vec<Uint128>,
}

impl ActivityBooster {
    pub fn new(
        assigned_amount: Uint128,
        distribution_amounts: Vec<Uint128>,
        drop_booster_reward_amount: Uint128,
        multiplier: Decimal,
    ) -> ActivityBooster {
        let reward_amount = drop_booster_reward_amount * multiplier;
        let reward_amounts = split_uint128(
            reward_amount,
            &distribution_amounts,
        );

        ActivityBooster {
            assigned_amount,
            distributed_amount: Uint128::zero(),
            reward_amount,
            reward_amounts,
        }
    }

    pub fn left_amount(&self) -> Uint128 {
        self.assigned_amount.checked_sub(self.distributed_amount).unwrap()
    }

    pub fn reward_amount(&self) -> Uint128 {
        std::cmp::min(self.left_amount(), self.reward_amount)
    }

    pub fn boost(
        &mut self,
        participation: &mut Participation,
        distance: u64,
        total_reward_amount: Uint128,
    ) -> Uint128 {
        let amount = self.reward_amounts.get(distance as usize)
            .map_or(Uint128::zero(), |v| v.clone())
            .multiply_ratio(total_reward_amount, self.reward_amount);

        participation.activity_booster_reward_amount += amount;
        self.distributed_amount += amount;

        amount
    }

    pub fn to_response(&self) -> ActivityBoosterResponse {
        ActivityBoosterResponse {
            assigned_amount: self.assigned_amount,
            distributed_amount: self.distributed_amount,
            reward_amounts: self.reward_amounts.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlusBooster {
    pub assigned_amount: Uint128,
    pub distributed_amount: Uint128,
}

impl PlusBooster {
    pub fn new(assigned_amount: Uint128) -> PlusBooster {
        PlusBooster {
            assigned_amount,
            distributed_amount: Uint128::zero(),
        }
    }

    pub fn boost(
        &mut self,
        participation: &mut Participation,
        voting_power: Decimal,
    ) -> Uint128 {
        let amount = self.assigned_amount * voting_power;

        participation.plus_booster_reward_amount += amount;
        self.distributed_amount += amount;

        amount
    }

    pub fn to_response(&self) -> PlusBoosterResponse {
        PlusBoosterResponse {
            assigned_amount: self.assigned_amount,
            distributed_amount: self.distributed_amount,
        }
    }
}

const PARTICIPATION: Map<&Addr, Participation> = Map::new("participation");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Participation {
    pub actor_address: Addr,
    pub referrer_address: Option<Addr>,
    pub reward_amount: Uint128,
    pub participated_at: Timestamp,

    // booster state
    pub drop_booster_claimable: Vec<(u64, bool)>,
    pub drop_booster_distance_counts: Vec<(u64, Vec<(u64, u64)>)>,
    pub activity_booster_reward_amount: Uint128,
    pub plus_booster_reward_amount: Uint128,
}

impl Participation {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        PARTICIPATION.save(storage, &self.actor_address, self)
    }

    pub fn load(storage: &dyn Storage, actor_address: &Addr) -> StdResult<Participation> {
        PARTICIPATION.load(storage, actor_address)
    }

    pub fn may_load(storage: &dyn Storage, actor_address: &Addr) -> StdResult<Option<Participation>> {
        PARTICIPATION.may_load(storage, actor_address)
    }

    pub fn load_referrers(&self, storage: &dyn Storage, distance_limit: usize) -> StdResult<Vec<Participation>> {
        let mut result = vec![];

        let mut referrer = self.referrer_address.clone();
        for _ in 0..distance_limit {
            if referrer.is_none() {
                break;
            }

            let referrer_participation = Self::may_load(storage, referrer.as_ref().unwrap())?;
            if referrer_participation.is_none() {
                break;
            }
            let referrer_participation = referrer_participation.unwrap();
            referrer = referrer_participation.referrer_address.clone();
            result.push(referrer_participation)
        }

        Ok(result)
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<Addr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Participation>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after.map(|v| Bound::exclusive(v.as_str().as_bytes()));
        let (min, max, order_by) = match order_by {
            Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
            _ => (None, start_after, OrderBy::Desc),
        };

        PARTICIPATION
            .range(storage, min, max, order_by.into())
            .take(limit)
            .map(|item| {
                let (_, v) = item?;
                Ok(v)
            })
            .collect()
    }

    pub fn increase_distance_count(&mut self, booster_id: u64, distance: u64) {
        match self.drop_booster_distance_counts.iter_mut().find(|(id, _)| *id == booster_id) {
            Some((_, distance_counts)) => {
                match distance_counts.iter_mut().find(|(d, _)| *d == distance) {
                    Some((_, count)) => *count += 1,
                    None => distance_counts.push((distance, 1)),
                }
            }
            None => self.drop_booster_distance_counts.push((booster_id, vec![(distance, 1)])),
        }
    }

    pub fn drop_booster_distance_counts(&self, booster_id: u64) -> Vec<(u64, u64)> {
        let mut result: Vec<(u64, u64)> = vec![];

        for (idx, distance_counts) in self.drop_booster_distance_counts.iter() {
            if *idx > booster_id {
                continue;
            }

            for (distance, count) in distance_counts.iter() {
                match result.iter_mut().find(|(d, _)| *d == *distance) {
                    Some(value) => value.1 += *count,
                    None => result.push((*distance, *count)),
                }
            }
        }

        result
    }

    pub fn has_reward(&self) -> bool {
        !self.reward_amount.is_zero()
    }

    pub fn has_booster_reward(&self, booster_id: u64) -> bool {
        self.has_drop_booster_reward(booster_id)
            || self.has_activity_booster_reward()
            || self.has_plus_booster_reward()
    }

    fn has_drop_booster_reward(&self, booster_id: u64) -> bool {
        for (id, claimable) in self.drop_booster_claimable.iter() {
            if *id > booster_id {
                continue;
            }

            if *claimable {
                return true;
            }
        }

        false
    }

    fn has_activity_booster_reward(&self) -> bool {
        !self.activity_booster_reward_amount.is_zero()
    }

    fn has_plus_booster_reward(&self) -> bool {
        !self.plus_booster_reward_amount.is_zero()
    }

    pub fn calc_drop_booster_amount(
        &self,
        storage: &dyn Storage,
        recent_booster_id: u64,
    ) -> StdResult<Uint128> {
        let mut result = Uint128::zero();

        for (id, claimable) in self.drop_booster_claimable.iter() {
            if !claimable {
                continue;
            }

            let booster = if recent_booster_id == *id {
                Booster::load(storage, *id)?
            } else {
                Booster::load_prev(storage, *id)?
            };

            result += booster.drop_booster.calc_reward_amount(self, *id);
        }

        Ok(result)
    }

    pub fn receive_reward(
        &mut self,
        campaign_state: &mut CampaignState,
    ) -> StdResult<Uint128> {
        let amount = self.reward_amount;

        self.reward_amount = Uint128::zero();
        campaign_state.locked_balance = campaign_state.locked_balance.checked_sub(amount)?;

        Ok(amount)
    }

    pub fn receive_booster_reward(
        &mut self,
        storage: &mut dyn Storage,
        recent_booster_id: u64,
    ) -> StdResult<Uint128> {
        let mut reward_amount = Uint128::zero();
        if self.has_drop_booster_reward(recent_booster_id) {
            reward_amount += self.receive_drop_booster(storage, recent_booster_id)?;
        }

        if self.has_activity_booster_reward() {
            reward_amount += self.receive_activity_booster();
        }

        if self.has_plus_booster_reward() {
            reward_amount += self.receive_plus_booster();
        }

        Ok(reward_amount)
    }

    fn receive_drop_booster(
        &mut self,
        storage: &mut dyn Storage,
        recent_booster_id: u64,
    ) -> StdResult<Uint128> {
        let mut result = Uint128::zero();

        let mut claimable_ids: Vec<u64> = vec![];
        for (id, claimable) in self.drop_booster_claimable.iter_mut() {
            if !*claimable {
                continue;
            }

            *claimable = false;
            claimable_ids.push(*id);
        }

        for id in claimable_ids { //iter_mut 로 이미 borrow 한 상태라서 calc_reward_amount 를 호출할 수 없어 반복문 분리함.
            let mut booster = if recent_booster_id == id {
                Booster::load(storage, id)?
            } else {
                Booster::load_prev(storage, id)?
            };
            let amount = booster.drop_booster.calc_reward_amount(self, id);
            booster.drop_booster.spent_amount += amount;
            booster.save(storage)?;

            result += amount;
        }

        Ok(result)
    }

    fn receive_activity_booster(&mut self) -> Uint128 {
        let amount = self.activity_booster_reward_amount;

        self.activity_booster_reward_amount = Uint128::zero();

        amount
    }

    fn receive_plus_booster(&mut self) -> Uint128 {
        let amount = self.plus_booster_reward_amount;

        self.plus_booster_reward_amount = Uint128::zero();

        amount
    }
}

pub fn load_global_campaign_config(
    querier: &QuerierWrapper,
    campaign_manager: &Addr,
) -> StdResult<valkyrie::campaign_manager::query_msgs::CampaignConfigResponse> {
    querier.query_wasm_smart(
        campaign_manager,
        &valkyrie::campaign_manager::query_msgs::QueryMsg::CampaignConfig {},
    )
}

pub fn load_voting_power(
    querier: &QuerierWrapper,
    governance: &Addr,
    staker_address: &Addr,
) -> Decimal {
    let response: StdResult<VotingPowerResponse> = querier.query_wasm_smart(
        governance,
        &valkyrie::governance::query_msgs::QueryMsg::VotingPower {
            address: staker_address.to_string(),
        },
    );

    response.map_or(Decimal::zero(), |v| v.voting_power)
}
