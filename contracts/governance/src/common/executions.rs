use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::governance::execute_msgs::ContractConfigInitMsg;

use super::states::ContractConfig;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ContractConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    ContractConfig {
        address: env.contract.address,
        governance_token: deps.api.addr_validate(&msg.governance_token)?,
    }.save(deps.storage)?;

    // Response
    Ok(Response::default())
}
