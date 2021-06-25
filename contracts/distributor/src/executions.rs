use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::states::{ContractConfig, DistributorInfo};

pub fn add_distributor(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    distributor: String,
    spend_limit: Uint128,
) -> ContractResult<Response> {
    // only governance contract can execute this message
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let distributor_addr = deps.api.addr_validate(&distributor)?;
    if DistributorInfo::load(deps.storage, &distributor_addr).is_ok() {
        return Err(ContractError::AlreadyExists {});
    }

    let distributor_info = DistributorInfo {
        distributor: distributor_addr,
        spend_limit,
    };

    distributor_info.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "add_distributor"),
            attr("distributor", distributor),
            attr("spend_limit", spend_limit),
        ],
        data: None,
    })
}

pub fn remove_distributor(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    distributor: String,
) -> ContractResult<Response> {
    // only governance contract can execute this message
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Ok(distributor_info) =
        DistributorInfo::load(deps.storage, &deps.api.addr_validate(&distributor)?)
    {
        distributor_info.remove(deps.storage);
    } else {
        return Err(ContractError::NotFound {});
    }

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "remove_distributor"),
            attr("distributor", distributor),
        ],
        data: None,
    })
}

pub fn spend(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    let mut distributor_info: DistributorInfo = DistributorInfo::load(deps.storage, &info.sender)?;
    if distributor_info.spend_limit < amount {
        return Err(ContractError::ExceedLimit {});
    }

    distributor_info.spend_limit = distributor_info.spend_limit.checked_sub(amount)?;
    if distributor_info.spend_limit.is_zero() {
        distributor_info.remove(deps.storage);
    } else {
        distributor_info.save(deps.storage)?;
    }

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_config.token_contract.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: deps.api.addr_validate(&recipient)?.to_string(),
                amount,
            })?,
            send: vec![],
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "spend"),
            attr("distributor", info.sender.as_str()),
            attr("recipient", recipient),
            attr("amount", amount),
        ],
        data: None,
    })
}
