use cosmwasm_std::{Decimal, Deps, Env, Uint64};

use valkyrie::common::ContractResult;
use valkyrie::governance::models::VoteInfoMsg;
use valkyrie::governance::query_msgs::{
    StakerStateResponse, StakingConfigResponse, StakingStateResponse, VotingPowerResponse,
};

use crate::common::states::load_contract_available_balance;
use crate::staking::states::StakingConfig;

use super::states::{StakerState, StakingState};

pub fn get_staking_config(deps: Deps, _env: Env) -> ContractResult<StakingConfigResponse> {
    let staking_config = StakingConfig::load(deps.storage)?;

    Ok(StakingConfigResponse {
        withdraw_delay: Uint64::from(staking_config.withdraw_delay),
    })
}

pub fn get_staking_state(deps: Deps, _env: Env) -> ContractResult<StakingStateResponse> {
    let staking_state = StakingState::load(deps.storage)?;

    Ok(StakingStateResponse {
        total_share: staking_state.total_share,
    })
}

pub fn get_staker_state(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<StakerStateResponse> {
    let address = deps.api.addr_validate(&address)?;
    let mut staker_state = StakerState::load(deps.storage, &address)?;

    let contract_available_balance = load_contract_available_balance(deps.clone())?;
    let balance = staker_state.load_balance(deps.storage, contract_available_balance)?;

    staker_state.clean_votes(deps.storage);

    let votes = staker_state
        .votes
        .iter()
        .map(|(poll_id, vote)| {
            let msg = VoteInfoMsg {
                voter: vote.voter.to_string(),
                option: vote.option.clone(),
                amount: vote.amount,
            };

            (*poll_id, msg)
        })
        .collect();

    Ok(StakerStateResponse {
        balance,
        share: staker_state.share,
        votes,
    })
}

pub fn get_voting_power(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<VotingPowerResponse> {
    let address = deps.api.addr_validate(&address)?;
    let staking_state: StakingState = StakingState::load(deps.storage)?;
    let staker_state: StakerState = StakerState::load(deps.storage, &address)?;

    Ok(VotingPowerResponse {
        voting_power: Decimal::from_ratio(staker_state.share, staking_state.total_share),
    })
}
