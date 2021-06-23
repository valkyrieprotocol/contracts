use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage, StdResult, StdError, Timestamp, QuerierWrapper};
use cw_storage_plus::{Item, Map, Bound};
use cw20::Denom;
use valkyrie::utils::find_mut_or_push;
use valkyrie::common::OrderBy;
use valkyrie::governance::query_msgs::ValkyrieConfigResponse;


const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admin: Addr,
    pub governance: Addr,
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
    pub cumulative_distribution_amount: Vec<(Denom, u128)>, //todo: Map 으로 변경?
    pub locked_balance: Vec<(Denom, u128)>,
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

    pub fn is_active(&self, storage: &dyn Storage, querier: &QuerierWrapper, block_height: u64) -> StdResult<bool> {
        if !self.active_flag {
            return Ok(false)
        }

        let config = ContractConfig::load(storage)?;
        let valkyrie_config = load_valkyrie_config(querier, &config.governance)?;

        //TODO: deactivate_period 를 그냥 campaign 에서 관리할까?
        Ok(valkyrie_config.campaign_deactivate_period.u64() + self.last_active_block.unwrap_or_default() >= block_height)
    }

    pub fn is_pending(&self) -> bool {
        self.last_active_block.is_none()
    }

    pub fn plus_distribution(&mut self, denom: Denom, amount: u128) {
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

    pub fn locked_balance(&self, denom: Denom) -> u128 {
        for (locked_denom, balance) in self.locked_balance.iter() {
            if denom.eq(locked_denom) {
                return *balance
            }
        }

        0u128
    }

    pub fn unlock_balance(&mut self, denom: Denom, amount: u128) -> StdResult<u128>{
        let balance = self.locked_balance.iter_mut().find(|v| v.0 == denom);

        if balance.is_none() {
            return Err(StdError::generic_err("Insufficient balance"));
        }

        let balance = balance.unwrap();
        balance.1 -= amount;

        Ok(balance.1)
    }
}

// pub fn is_active(storage: &dyn Storage, querier: &QuerierWrapper, block_height: u64) -> StdResult<bool> {
//     CampaignState::load(storage)?.is_active(storage, querier, block_height)
// }

pub fn is_pending(storage: &dyn Storage) -> StdResult<bool> {
    Ok(CampaignState::load(storage)?.is_pending())
}


const DISTRIBUTION_CONFIG: Item<DistributionConfig> = Item::new("distribution_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionConfig {
    pub denom: Denom,
    pub amounts: Vec<u128>,
}

impl DistributionConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        DISTRIBUTION_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<DistributionConfig> {
        DISTRIBUTION_CONFIG.load(storage)
    }
}


const PARTICIPATION: Map<&Addr, Participation> = Map::new("participation");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Participation {
    pub actor_address: Addr,
    pub referrer_address: Option<Addr>,
    pub rewards: Vec<(Denom, u128)>,
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

        PARTICIPATION.range(storage, min, max, order_by.into())
            .take(limit)
            .map(|item| {
                let (_, v) = item?;
                Ok(v)
            })
            .collect()
    }

    pub fn plus_reward(&mut self, denom: Denom, amount: u128) {
        find_mut_or_push(
            &mut self.rewards,
            |v| v.0 == denom.clone(),
            || (denom.clone(), amount),
            |v| v.1 += amount,
        );
    }
}

pub fn load_valkyrie_config(querier: &QuerierWrapper, governance: &Addr) -> StdResult<ValkyrieConfigResponse> {
    querier.query_wasm_smart(
        governance,
        &valkyrie::governance::query_msgs::QueryMsg::ValkyrieConfig {},
    )
}
