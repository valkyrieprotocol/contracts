use cosmwasm_std::{Deps, StdResult};

use crate::executions::before_share_change;
use crate::states::{StakerInfo, State};
use valkyrie::lp_staking::query_msgs::StakerInfoResponse;

pub fn query_staker_info(deps: Deps, staker_addr: String) -> StdResult<StakerInfoResponse> {
    let mut staker_info: StakerInfo =
        StakerInfo::load_or_default(deps.storage, &deps.api.addr_validate(staker_addr.as_str())?)?;

    let state = State::load(deps.storage)?;
    before_share_change(&state, &mut staker_info)?;

    Ok(StakerInfoResponse {
        staker_addr: staker_addr.as_str().to_string(),
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}
