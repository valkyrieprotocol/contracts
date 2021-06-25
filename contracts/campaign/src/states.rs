use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, QuerierWrapper, StdError, StdResult, Storage, Timestamp, Uint128};
use cw20::Denom;
use cw_storage_plus::{Bound, Item, Map};
use valkyrie::common::OrderBy;
use valkyrie::governance::query_msgs::{ValkyrieConfigResponse, VotingPowerResponse};
use valkyrie::utils::find_mut_or_push;

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: Addr,
    pub governance: Addr,
    pub distributor: Addr,
    pub token_contract: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admin.eq(address)
    }

    pub fn is_distributor(&self, address: &Addr) -> bool {
        self.distributor.eq(address)
    }

    // pub fn is_governance(&self, address: &Addr) -> bool {
    //     self.governance.eq(address)
    // }
}

pub fn is_admin(storage: &dyn Storage, address: &Addr) -> bool {
    ContractConfig::load(storage).unwrap().is_admin(address)
}

// pub fn is_governance(storage: &dyn Storage, address: &Addr) -> bool {
//     ContractConfig::load(storage).unwrap().is_governance(address)
// }

const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInfo {
    pub title: String,
    pub description: String,
    pub url: String,
    pub parameter_key: String,
    pub creator: Addr,
    pub created_at: Timestamp,
    pub created_block: u64,
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
    pub cumulative_distribution_amount: Vec<(Denom, Uint128)>,
    pub locked_balance: Vec<(Denom, Uint128)>,
    pub active_flag: bool,
    pub last_active_block: Option<u64>,
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
        let valkyrie_config = load_valkyrie_config(querier, &config.governance)?;

        //TODO: deactivate_period 를 그냥 campaign 에서 관리할까?
        Ok(valkyrie_config.campaign_deactivate_period.u64()
            + self.last_active_block.unwrap_or_default()
            >= block_height)
    }

    pub fn is_pending(&self) -> bool {
        self.last_active_block.is_none()
    }

    pub fn plus_distribution(&mut self, denom: Denom, amount: Uint128) {
        find_mut_or_push(
            &mut self.cumulative_distribution_amount,
            |v| v.0 == denom,
            || (denom.clone(), amount),
            |v| v.1 += amount,
        );

        find_mut_or_push(
            &mut self.locked_balance,
            |v| v.0 == denom,
            || (denom.clone(), amount),
            |v| v.1 += amount,
        );
    }

    pub fn locked_balance(&self, denom: Denom) -> Uint128 {
        for (locked_denom, balance) in self.locked_balance.iter() {
            if denom.eq(locked_denom) {
                return *balance;
            }
        }

        Uint128::zero()
    }

    pub fn unlock_balance(&mut self, denom: Denom, amount: Uint128) -> StdResult<Uint128> {
        let balance = self.locked_balance.iter_mut().find(|v| v.0 == denom);

        if balance.is_none() {
            return Err(StdError::generic_err("Insufficient balance"));
        }

        let balance = balance.unwrap();
        balance.1 = balance.1.checked_sub(amount)?;

        Ok(balance.1)
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
        DISTRIBUTION_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<DistributionConfig> {
        DISTRIBUTION_CONFIG.load(storage)
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
    pub drop_booster_amount: Uint128,
    pub drop_booster_left_amount: Uint128,
    pub drop_booster_participations: u64,
    pub activity_booster_amount: Uint128,
    pub activity_booster_left_amount: Uint128,
    pub plus_booster_amount: Uint128,
    pub plus_booster_left_amount: Uint128,
}

impl BoosterState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        BOOSTER_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<BoosterState> {
        BOOSTER_STATE.load(storage)
    }

    pub fn remove(storage: &mut dyn Storage) {
        BOOSTER_STATE.remove(storage)
    }

    pub fn compute_drop_booster(&self) -> Uint128 {
        std::cmp::min(
            self.drop_booster_left_amount,
            Uint128::from(
                self.drop_booster_amount.u128() / self.drop_booster_participations as u128,
            ),
        )
    }

    pub fn spend_drop_booster(&mut self, amount: Uint128) -> StdResult<()> {
        self.drop_booster_left_amount = self.drop_booster_left_amount.checked_sub(amount)?;
        Ok(())
    }

    pub fn compute_activity_booster(&self) -> Uint128 {
        std::cmp::min(
            self.activity_booster_left_amount,
            Uint128::from(self.compute_drop_booster().u128() * 5u128 / 4u128),
        )
    }

    pub fn spend_activity_booster(&mut self, amount: Uint128) -> StdResult<()> {
        self.activity_booster_left_amount =
            self.activity_booster_left_amount.checked_sub(amount)?;
        Ok(())
    }

    pub fn compute_plus_booster(
        &self,
        querier: &QuerierWrapper,
        governance: &Addr,
        address: &Addr,
    ) -> StdResult<Uint128> {
        Ok(std::cmp::min(
            self.plus_booster_left_amount,
            self.plus_booster_amount
                * load_voting_power(querier, governance, address)?.voting_power,
        ))
    }

    pub fn spend_plus_booster(&mut self, amount: Uint128) -> StdResult<()> {
        self.plus_booster_left_amount = self.plus_booster_left_amount.checked_sub(amount)?;
        Ok(())
    }

    pub fn compute_and_spend_participate_booster(
        storage: &mut dyn Storage,
        querier: &QuerierWrapper,
        governance: &Addr,
        address: &Addr,
    ) -> StdResult<(Uint128, Uint128, bool)> {
        if let Some(mut booster_state) = BOOSTER_STATE.may_load(storage)? {
            let activity_booster = booster_state.compute_activity_booster();
            let plus_booster = booster_state.compute_plus_booster(querier, governance, address)?;

            booster_state.spend_activity_booster(activity_booster)?;
            booster_state.spend_plus_booster(plus_booster)?;
            booster_state.save(storage)?;

            Ok((activity_booster, plus_booster, false))
        } else {
            Ok((Uint128::zero(), Uint128::zero(), true))
        }
    }

    pub fn compute_and_spend_drop_booster(storage: &mut dyn Storage) -> StdResult<Uint128> {
        if let Some(mut booster_state) = BOOSTER_STATE.may_load(storage)? {
            let drop_booster = booster_state.compute_drop_booster();

            booster_state.spend_drop_booster(drop_booster)?;
            booster_state.save(storage)?;
            Ok(drop_booster)
        } else {
            Ok(Uint128::zero())
        }
    }
}

const PARTICIPATION: Map<&Addr, Participation> = Map::new("participation");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Participation {
    pub actor_address: Addr,
    pub referrer_address: Option<Addr>,
    pub rewards: Vec<(Denom, Uint128)>,

    // booster state
    pub booster_rewards: Uint128,
    pub drop_booster_claimable: bool,
}

impl Participation {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        PARTICIPATION.save(storage, &self.actor_address, self)
    }

    pub fn load(storage: &dyn Storage, actor_address: &Addr) -> StdResult<Participation> {
        PARTICIPATION.load(storage, actor_address)
    }

    pub fn query<'a>(
        storage: &'a dyn Storage,
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

    pub fn plus_reward(&mut self, denom: Denom, amount: Uint128) {
        find_mut_or_push(
            &mut self.rewards,
            |v| v.0 == denom.clone(),
            || (denom.clone(), amount),
            |v| v.1 += amount,
        );
    }
}

pub fn load_valkyrie_config(
    querier: &QuerierWrapper,
    governance: &Addr,
) -> StdResult<ValkyrieConfigResponse> {
    querier.query_wasm_smart(
        governance,
        &valkyrie::governance::query_msgs::QueryMsg::ValkyrieConfig {},
    )
}

pub fn load_voting_power(
    querier: &QuerierWrapper,
    governance: &Addr,
    staker_address: &Addr,
) -> StdResult<VotingPowerResponse> {
    querier.query_wasm_smart(
        governance,
        &valkyrie::governance::query_msgs::QueryMsg::VotingPower {
            address: staker_address.to_string(),
        },
    )
}
