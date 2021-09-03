use cosmwasm_std::{Deps, Env, StdResult};

use crate::states::{Config, StakerInfo, State};
use valkyrie::lp_staking::query_msgs::{ConfigResponse, StateResponse, StakerInfoResponse};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = Config::load(deps.storage)?;
    let resp = ConfigResponse {
        token: config.token.to_string(),
        pair: config.pair.to_string(),
        lp_token: config.lp_token.to_string(),
        distribution_schedule: config.distribution_schedule,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps, block_height: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = State::load(deps.storage)?;
    if let Some(block_height) = block_height {
        let config: Config = Config::load(deps.storage)?;
        state.compute_reward(&config, block_height);
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
    })
}

pub fn query_staker_info(deps: Deps, env: Env, staker: String) -> StdResult<StakerInfoResponse> {
    let block_height = env.block.height;
    let staker_raw = deps.api.addr_validate(&staker.as_str())?;

    let mut staker_info: StakerInfo = StakerInfo::load_or_default(deps.storage, &staker_raw)?;

    let config: Config = Config::load(deps.storage)?;
    let mut state: State = State::load(deps.storage)?;

    state.compute_reward(&config, block_height);
    staker_info.compute_staker_reward(&state)?;

    Ok(StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}
