use cosmwasm_std::{Addr, StdResult, Storage, Deps, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use valkyrie::cw20::query_cw20_balance;
use crate::poll::states::PollState;


const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub address: Addr, // contract address
    pub governance_token: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_governance_token(&self, address: &Addr) -> bool {
        self.governance_token.eq(address)
    }
}

pub fn load_available_balance(deps: Deps) -> StdResult<Uint128> {
    let contract_config = ContractConfig::load(deps.storage)?;
    let contract_balance = query_cw20_balance(
        &deps.querier,
        &contract_config.governance_token,
        &contract_config.address,
    )?;
    let poll_state = PollState::load(deps.storage)?;
    let available_balance = contract_balance.checked_sub(poll_state.total_deposit)?;

    Ok(available_balance)
}