use cosmwasm_std::{Addr, Env, StdResult, Storage};

pub fn migrate(
    _storage: &mut dyn Storage,
    _env: &Env,
    _router: &Addr,
) -> StdResult<()> {
    Ok(())
}