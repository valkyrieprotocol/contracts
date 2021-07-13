use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, attr};

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
    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "instantiate"),
        ],
        data: None,
    })
}
