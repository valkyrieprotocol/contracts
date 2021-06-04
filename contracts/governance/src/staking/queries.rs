use cosmwasm_std::{Addr, Deps, Env, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::governance::enumerations::PollStatus;
use valkyrie::governance::models::{StakerStateResponse, StakingStateResponse};

use crate::common::state::ContractConfig;
use crate::cw20::load_cw20_balance;
use crate::poll::state::{Poll, PollState};

use super::state::{StakerState, StakingState};

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
    env: Env,
    address: Addr,
) -> ContractResult<StakerStateResponse> {
    let contract_config = ContractConfig::load(deps.storage)?;
    let poll_state = PollState::load(deps.storage)?;
    let staking_state = StakingState::load(deps.storage)?;
    let mut staker_state = StakerState::load(
        deps.storage,
        &deps.api.addr_canonicalize(address.as_str())?,
    )?;

    staker_state.locked_balance.retain(|(poll_id, _)| {
        let poll = Poll::load(deps.storage, &poll_id).unwrap();

        poll.status == PollStatus::InProgress
    });

    let contract_balance = load_cw20_balance(
        &deps.querier,
        &deps.api.addr_humanize(&contract_config.token_contract)?,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
    )?;
    let total_balance = contract_balance.checked_sub(poll_state.total_deposit)?;
    let balance = if !staking_state.total_share.is_zero() {
        staker_state.share.multiply_ratio(total_balance, staking_state.total_share)
    } else {
        Uint128::zero()
    };

    Ok(
        StakerStateResponse {
            balance,
            share: staker_state.share,
            locked_balance: staker_state.locked_balance,
        }
    )
}