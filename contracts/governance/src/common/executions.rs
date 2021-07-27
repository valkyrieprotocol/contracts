use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::governance::execute_msgs::ContractConfigInitMsg;

use super::states::ContractConfig;
use valkyrie::utils::make_response;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ContractConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    ContractConfig {
        address: env.contract.address,
        governance_token: deps.api.addr_validate(&msg.governance_token)?,
    }.save(deps.storage)?;

    Ok(response)
}
