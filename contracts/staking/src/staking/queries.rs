use cosmwasm_std::{Deps, Env, StdResult};

use valkyrie::staking::{ConfigResponse, StakerInfoResponse, StateResponse};

use crate::staking::states::{
    compute_reward, compute_staker_reward, read_staker_info, Config, StakerInfo, State, CONFIG,
    STATE,
};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        valkyrie_token: config.valkyrie_token.to_string(),
        staking_token: config.liquidity_token.to_string(),
        distribution_schedule: config.distribution_schedule,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps, block_height: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = STATE.load(deps.storage)?;
    if let Some(block_height) = block_height {
        let config: Config = CONFIG.load(deps.storage)?;
        compute_reward(&config, &mut state, block_height);
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

    let mut staker_info: StakerInfo = read_staker_info(&deps, &staker_raw)?;

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    compute_reward(&config, &mut state, block_height);
    compute_staker_reward(&state, &mut staker_info)?;

    Ok(StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}
