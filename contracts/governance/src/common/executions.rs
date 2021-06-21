use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::governance::messages::ContractConfigInitMsg;

use super::states::ContractConfig;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ContractConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let config = ContractConfig {
        address: env.contract.address,
        token_contract: deps.api.addr_validate(&msg.token_contract)?,
    };

    config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}
