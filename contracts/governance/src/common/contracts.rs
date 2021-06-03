use cosmwasm_std::{Addr, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::governance::messages::{InstantiateMsg, ContractConfigInitMsg};

use crate::errors::ContractError;
use crate::poll::state::PollConfig;
use crate::staking::state::StakingConfig;

use super::state::ContractConfig;

pub fn instantiate(
    deps: &DepsMut,
    _env: &Env,
    info: &MessageInfo,
    msg: ContractConfigInitMsg,
) -> ContractResult<Response> {
    let config = ContractConfig {
        admin: deps.api.addr_canonicalize(info.sender.as_str())?,
        token_contract: deps.api.addr_canonicalize(msg.token_contract.as_str())?,
        boost_contract: None,
    };

    ContractConfig::singleton(deps.storage).save(&config)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<Addr>,
    boost_contract: Option<Addr>,
) -> ContractResult<Response> {
    ContractConfig::singleton(deps.storage).update(|mut config| {
        if !config.is_admin(deps.api.addr_canonicalize(info.sender.as_str())?) {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(admin) = admin {
            config.admin = deps.api.addr_canonicalize(admin.as_str())?
        }

        if let Some(boost_contract) = boost_contract {
            config.boost_contract = Some(deps.api.addr_canonicalize(boost_contract.as_str())?)
        }

        Ok(config)
    })?;

    Ok(Response::default())
}