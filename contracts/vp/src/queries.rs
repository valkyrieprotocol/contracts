use cosmwasm_std::{Decimal, Deps, StdResult};
use crate::state::{Config, State, SwapState};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    Config::load(deps.storage)
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    State::load_or_default(deps.storage)
}

pub fn query_swap_state(deps: Deps, ratio:Decimal) -> StdResult<SwapState> {
    SwapState::load_or_default(deps.storage, ratio)
}