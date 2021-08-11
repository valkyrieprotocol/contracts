use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Storage, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg, Denom};
use cw_storage_plus::{Item, Map};
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


const REQUIREMENT: Item<Requirement> = Item::new("requirement");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Requirement {
    pub min_token_balances: Vec<(Denom, Uint128)>,
    pub min_luna_staking: Uint128,
    pub collateral_denom: Option<Denom>,
    pub collateral_amount: Uint128,
    pub collateral_lock_period: u64,
}

impl Requirement {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        REQUIREMENT.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Requirement> {
        REQUIREMENT.load(storage)
    }

    pub fn require_collateral(&self) -> bool {
        self.collateral_denom.is_some() && !self.collateral_amount.is_zero()
    }

    pub fn is_satisfy_requirements(&self, querier: &Querier, actor: &Addr, collateral_balance: Uint128) -> StdResult<(bool, String)> {
        let result = self.is_satisfy_luna_staking_amount(&querier, &actor)?;
        if !result.0 {
            return Ok(result);
        }

        let result = self.is_satisfy_min_token_balances(&querier, &actor)?;
        if !result.0 {
            return Ok(result);
        }

        let result = self.is_satisfy_collateral(collateral_balance)?;
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

    fn is_satisfy_collateral(&self, collateral_balance: Uint128) -> StdResult<(bool, String)> {
        if !self.require_collateral() {
            return Ok((true, String::default()));
        }

        if collateral_balance < self.collateral_amount {
            return Ok((false, format!(
                "Insufficient collateral balance (required: {}, current: {})",
                self.collateral_amount.to_string(),
                collateral_balance.to_string(),
            )));
        }

        Ok((true, String::default()))
    }
}


const COLLATERALS: Map<&Addr, Collateral> = Map::new("collateral");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Collateral {
    pub owner: Addr,
    pub deposit_amount: Uint128,
    pub locked_amounts: Vec<(Uint128, u64)>,
}

impl Collateral {
    pub fn new(owner: Addr) -> Collateral {
        Collateral {
            owner,
            deposit_amount: Uint128::zero(),
            locked_amounts: vec![],
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        COLLATERALS.save(storage, &self.owner, self)
    }

    pub fn load(storage: &dyn Storage, owner: &Addr) -> StdResult<Collateral> {
        COLLATERALS.load(storage, owner)
    }

    pub fn load_or_new(storage: &dyn Storage, owner: &Addr) -> StdResult<Collateral> {
        Ok(COLLATERALS.may_load(storage, owner)
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

fn denom_to_string(denom: &Denom) -> String {
    match denom {
        Denom::Native(denom) => denom.to_string(),
        Denom::Cw20(address) => address.to_string(),
    }
}
