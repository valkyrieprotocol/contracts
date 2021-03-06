use cosmwasm_std::{Decimal, Deps, Env};

use valkyrie::common::ContractResult;
use valkyrie::governance::models::VoteInfoMsg;
use valkyrie::governance::query_msgs::{AllStakersResponse, StakerInfoResponse, StakerStateResponse, StakingStateResponse, VotingPowerResponse};

use crate::common::states::load_available_balance;

use super::states::{StakerState, StakingState};
use crate::staking::states::StakingConfig;


pub fn get_staking_config(deps: Deps, _env: Env) -> ContractResult<StakingConfig> {
    Ok(StakingConfig::load(deps.storage)?)
}

pub fn get_staking_state(deps: Deps, env: Env) -> ContractResult<StakingStateResponse> {
    let staking_state = StakingState::load(deps.storage)?;

    Ok(StakingStateResponse {
        total_share: staking_state.total_share,
        total_balance: load_available_balance(deps.clone(), env.block.height)?,
    })
}

pub fn get_staker_state(
    deps: Deps,
    env: Env,
    address: String,
) -> ContractResult<StakerStateResponse> {
    let address = deps.api.addr_validate(&address)?;
    let staker_state = StakerState::may_load(deps.storage, &address)?;

    if staker_state.is_none() {
        return Ok(StakerStateResponse::default())
    }

    let mut staker_state = staker_state.unwrap();

    let contract_available_balance = load_available_balance(deps.clone(), env.block.height)?;
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

pub fn get_all_stakers(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> ContractResult<AllStakersResponse> {
    Ok(AllStakersResponse {
        stakers: StakerState::load_all(deps.storage, start_after, limit)?.iter()
            .map(|s| StakerInfoResponse {
                address: s.address.to_string(),
                share: s.share,
            })
            .collect(),
    })
}
