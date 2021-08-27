use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Storage, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg, Denom};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie_qualifier::QualifiedContinueOption;
use valkyrie::campaign::query_msgs::ActorResponse;

const QUALIFIER_CONFIG: Item<QualifierConfig> = Item::new("qualifier_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualifierConfig {
    pub admin: Addr,
    pub continue_option_on_fail: QualifiedContinueOption,
}

impl QualifierConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        QUALIFIER_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<QualifierConfig> {
        QUALIFIER_CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admin == *address
    }
}

pub fn is_admin(storage: &dyn Storage, address: &Addr) -> StdResult<bool> {
    QualifierConfig::load(storage).map(|c| c.is_admin(address))
}


const REQUIREMENT: Item<Requirement> = Item::new("requirement");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Requirement {
    pub min_token_balances: Vec<(Denom, Uint128)>,
    pub min_luna_staking: Uint128,
    pub participation_limit: u64,
}

impl Requirement {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        REQUIREMENT.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Requirement> {
        REQUIREMENT.load(storage)
    }

    pub fn is_satisfy_requirements(&self, querier: &Querier, campaign: &Addr, actor: &Addr) -> StdResult<(bool, String)> {
        let result = self.is_satisfy_luna_staking_amount(&querier, actor)?;
        if !result.0 {
            return Ok(result);
        }

        let result = self.is_satisfy_min_token_balances(&querier, actor)?;
        if !result.0 {
            return Ok(result);
        }

        let result = self.is_satisfy_participation_count(&querier, campaign, actor)?;
        if !result.0 {
            return Ok(result);
        }

        Ok((true, String::default()))
    }

    fn is_satisfy_luna_staking_amount(&self, querier: &Querier, actor: &Addr) -> StdResult<(bool, String)> {
        if self.min_luna_staking.is_zero() {
            return Ok((true, String::default()));
        }

        let current_luna_staking_amount = querier.load_luna_staking_amount(&actor)?;

        if current_luna_staking_amount < self.min_luna_staking {
            return Ok((false, format!(
                "Insufficient luna staking amount(required: {}, current: {})",
                self.min_luna_staking.to_string(),
                current_luna_staking_amount.to_string(),
            )));
        }

        Ok((true, String::default()))
    }

    fn is_satisfy_min_token_balances(&self, querier: &Querier, actor: &Addr) -> StdResult<(bool, String)> {
        for (denom, min_balance) in self.min_token_balances.iter() {
            if min_balance.is_zero() {
                continue;
            }

            let current_balance = querier.load_balance(denom, &actor)?;

            if current_balance < *min_balance {
                return Ok((false, format!(
                    "Insufficient token({}) balance (required: {}, current: {})",
                    denom_to_string(denom),
                    min_balance.to_string(),
                    current_balance.to_string(),
                )));
            }
        }

        Ok((true, String::default()))
    }

    fn is_satisfy_participation_count(&self, querier: &Querier, campaign: &Addr, actor: &Addr) -> StdResult<(bool, String)> {
        if self.participation_limit == 0 {
            return Ok((true, String::default()));
        }

        let participation_count = querier.load_participation_count(campaign, actor)?;

        if participation_count >= self.participation_limit {
            return Ok((false, format!(
                "Exceed participation limit(limit: {}, current: {})",
                self.participation_limit.to_string(),
                participation_count.to_string(),
            )));
        }

        Ok((true, String::default()))
    }
}


pub struct Querier<'a> {
    querier: &'a QuerierWrapper<'a>,
}

impl Querier<'_> {
    pub fn new<'a>(querier: &'a QuerierWrapper<'a>) -> Querier<'a> {
        Querier {
            querier
        }
    }

    pub fn load_balance(&self, denom: &Denom, address: &Addr) -> StdResult<Uint128> {
        match denom {
            Denom::Native(denom) => self.load_native_balance(denom, address),
            Denom::Cw20(token_contract) => self.load_cw20_balance(token_contract, address),
        }
    }

    pub fn load_luna_staking_amount(&self, address: &Addr) -> StdResult<Uint128> {
        let delegations = self.querier.query_all_delegations(address)?;

        let mut staking_amount = Uint128::zero();

        for delegation in delegations.iter() {
            if delegation.amount.denom == "uluna" {
                staking_amount += delegation.amount.amount;
            }
        }

        Ok(staking_amount)
    }

    fn load_native_balance(&self, denom: &String, address: &Addr) -> StdResult<Uint128> {
        Ok(self.querier.query_balance(address, denom)?.amount)
    }

    fn load_cw20_balance(&self, token_contract: &Addr, address: &Addr) -> StdResult<Uint128> {
        let balance: BalanceResponse = self.querier.query_wasm_smart(
            token_contract,
            &Cw20QueryMsg::Balance {
                address: address.to_string(),
            },
        )?;

        Ok(balance.balance)
    }

    fn load_participation_count(&self, campaign: &Addr, address: &Addr) -> StdResult<u64> {
        let actor: ActorResponse = self.querier.query_wasm_smart(
            campaign,
            &valkyrie::campaign::query_msgs::QueryMsg::Actor {
                address: address.to_string(),
            },
        )?;

        Ok(actor.participation_count)
    }
}

fn denom_to_string(denom: &Denom) -> String {
    match denom {
        Denom::Native(denom) => denom.to_string(),
        Denom::Cw20(address) => address.to_string(),
    }
}
