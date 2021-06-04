use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::governance::messages::ContractConfigInitMsg;

use super::state::ContractConfig;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ContractConfigInitMsg,
) -> ContractResult<Response> {
    let config = ContractConfig {
        admin: deps.api.addr_canonicalize(info.sender.as_str())?,
        token_contract: deps.api.addr_canonicalize(msg.token_contract.as_str())?,
        boost_contract: None,
    };

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<Addr>,
    boost_contract: Option<Addr>,
) -> ContractResult<Response> {
    let mut contract_config = ContractConfig::load(deps.storage)?;

    if !contract_config.is_admin(deps.api.addr_canonicalize(info.sender.as_str())?) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        contract_config.admin = deps.api.addr_canonicalize(admin.as_str())?
    }

    if let Some(boost_contract) = boost_contract {
        contract_config.boost_contract = Some(deps.api.addr_canonicalize(boost_contract.as_str())?)
    }

    contract_config.save(deps.storage)?;

    Ok(Response::default())
}