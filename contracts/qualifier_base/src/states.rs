use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Storage, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg, Denom};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie_qualifier::QualifiedContinueOption;

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


const QUALIFICATION_REQUIREMENT: Item<QualificationRequirement> = Item::new("qualification_requirement");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QualificationRequirement {
    pub min_token_balances: Vec<(Denom, Uint128)>,
    pub min_luna_staking: Uint128,
}

impl QualificationRequirement {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        QUALIFICATION_REQUIREMENT.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<QualificationRequirement> {
        QUALIFICATION_REQUIREMENT.load(storage)
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
}
