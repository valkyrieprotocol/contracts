use cw_storage_plus::{Item, Map};
use cosmwasm_std::{Addr, Storage, StdResult, Uint128, QuerierWrapper, Api, Env};
use valkyrie::common::OrderBy;
use valkyrie::fund_manager::query_msgs::{AllowancesResponse, AllowanceResponse, BalanceResponse};
use valkyrie::pagination::addr_range_option;
use valkyrie::cw20::query_cw20_balance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub admins: Vec<Addr>,
    pub managing_token: Addr,
    pub terraswap_router: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        self.admins.contains(address)
    }
}

const CONTRACT_STATE: Item<ContractState> = Item::new("contract-state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractState {
    pub remain_allowance_amount: Uint128,
}

impl ContractState {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractState> {
        CONTRACT_STATE.load(storage)
    }

    pub fn load_balance(
        &self,
        querier: &QuerierWrapper,
        api: &dyn Api,
        env: &Env,
        token_address: &Addr,
    ) -> StdResult<BalanceResponse> {
        let total_balance = query_cw20_balance(
            querier,
            api,
            token_address,
            &env.contract.address,
        )?;

        Ok(BalanceResponse {
            total_balance,
            allowance_amount: self.remain_allowance_amount,
            free_balance: total_balance.checked_sub(self.remain_allowance_amount)?,
        })
    }
}

const ALLOWANCE: Map<&Addr, Allowance> = Map::new("allowance");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Allowance {
    pub address: Addr,
    pub allowed_amount: Uint128,
    pub remain_amount: Uint128,
}

impl Allowance {
    pub fn default(address: &Addr) -> Allowance {
        Allowance {
            address: address.clone(),
            allowed_amount: Uint128::zero(),
            remain_amount: Uint128::zero(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        ALLOWANCE.save(storage, &self.address, self)
    }

    pub fn delete(&self, storage: &mut dyn Storage) {
        ALLOWANCE.remove(storage, &self.address)
    }

    pub fn save_or_delete(&self, storage: &mut dyn Storage) -> StdResult<()> {
        if self.remain_amount.is_zero() {
            self.delete(storage);
            Ok(())
        } else {
            self.save(storage)
        }
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<Allowance> {
        ALLOWANCE.load(storage, address)
    }

    pub fn may_load(storage: &dyn Storage, address: &Addr) -> StdResult<Option<Allowance>> {
        ALLOWANCE.may_load(storage, address)
    }

    pub fn load_or_default(storage: &dyn Storage, address: &Addr) -> StdResult<Allowance> {
        Ok(Self::may_load(storage, address)?.unwrap_or_else(|| Self::default(address)))
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<AllowancesResponse> {
        let range_option = addr_range_option(start_after, limit, order_by);

        let allowances = ALLOWANCE
            .range(storage, range_option.min, range_option.max, range_option.order_by)
            .take(range_option.limit)
            .map(|item| {
                let (_, allowance) = item?;

                Ok(AllowanceResponse {
                    address: allowance.address.to_string(),
                    allowed_amount: allowance.allowed_amount,
                    remain_amount: allowance.remain_amount,
                })
            })
            .collect::<StdResult<Vec<AllowanceResponse>>>()?;

        Ok(AllowancesResponse {
            allowances,
        })
    }

    pub fn increase(&mut self, amount: Uint128) {
        self.allowed_amount += amount;
        self.remain_amount += amount;
    }

    pub fn decrease(&mut self, amount: Uint128) -> StdResult<()> {
        self.allowed_amount = self.allowed_amount.checked_sub(amount.clone())?;
        self.remain_amount = self.remain_amount.checked_sub(amount)?;

        Ok(())
    }
}
