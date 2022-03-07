use cosmwasm_std::{DepsMut, Env, Response};
use valkyrie::campaign::execute_msgs::{MigrateMsg};
use valkyrie::common::{ContractResult};
use valkyrie::utils::{make_response};

use crate::states::*;

pub fn migrate(
    deps: DepsMut,
    env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response> {
    // Execute
    let mut response = make_response("migrate");

    response = response.add_attribute("chain_id", env.block.chain_id.clone());
    let mut campaign_state = CampaignState::load(deps.storage)?;
    campaign_state.chain_id = env.block.chain_id;
    campaign_state.save(deps.storage)?;

    Ok(response)
}