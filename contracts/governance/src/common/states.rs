use cosmwasm_std::{Addr, StdResult, Storage, Deps, Uint128, Env};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use valkyrie::cw20::query_cw20_balance;
use crate::poll::states::PollState;
use crate::staking::states::StakingState;


const CONTRACT_CONFIG: Item<ContractConfig> = Item::new("contract-config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfig {
    pub address: Addr, // contract address
    pub token_contract: Addr,
}

impl ContractConfig {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONTRACT_CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ContractConfig> {
        CONTRACT_CONFIG.load(storage)
    }

    pub fn is_token_contract(&self, address: &Addr) -> bool {
        self.token_contract.eq(address)
    }
}

// contract only managed by itself
pub fn is_admin(_storage: &dyn Storage, env: Env, address: &Addr) -> bool {
    // let contract_config = ContractConfig::load(storage)?;
    // contract_config.is_admin(address)

    env.contract.address.eq(address)
}

// 투표가 quorum 을 넘길경우 발의자에게 환불한다.
// quorum 을 넘기지 못해 환불되지 않는 경우 total_deposit 에서만 차감된다.
// contract balance 는 staking + poll deposit 이다.
// total_deposit 에서만 차감하고 실제 출금을 하지 않았기때문에 contract balance 가 더 많을 수도 있다.
pub fn load_contract_available_balance(deps: Deps) -> StdResult<Uint128> {
    let contract_config = ContractConfig::load(deps.storage)?;
    let contract_balance = query_cw20_balance(
        &deps.querier,
        deps.api,
        &contract_config.token_contract,
        &contract_config.address,
    )?;
    let poll_state = PollState::load(deps.storage)?;
    let staking_state = StakingState::load(deps.storage)?;
    let available_balance = contract_balance.checked_sub(poll_state.total_deposit)?
        .checked_sub(staking_state.unstaking_amount)?;

    Ok(available_balance)
}