use cosmwasm_std::{Addr, Deps, Env};

use valkyrie::common::ContractResult;
use valkyrie::governance::models::{StakerStateResponse, StakingStateResponse};

use crate::common::states::load_contract_available_balance;

use super::states::{StakerState, StakingState};

pub fn get_staking_state(
    deps: Deps,
    _env: Env,
) -> ContractResult<StakingStateResponse> {
    let staking_state = StakingState::load(deps.storage)?;

    Ok(
        StakingStateResponse {
            total_share: staking_state.total_share,
        }
    )
}

pub fn get_staker_state(
    deps: Deps,
    _env: Env,
    address: Addr,
) -> ContractResult<StakerStateResponse> {
    let mut staker_state = StakerState::load(deps.storage, &address)?;

    let contract_available_balance = load_contract_available_balance(deps.clone())?;
    let balance = staker_state.load_balance(deps.storage, contract_available_balance)?;

    staker_state.clean_votes(deps.storage);

    Ok(
        StakerStateResponse {
            balance,
            share: staker_state.share,
            votes: staker_state.votes,
        }
    )
}