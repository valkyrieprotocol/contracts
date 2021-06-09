use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::common::states::is_admin;
use crate::valkyrie::states::CampaignCode;

use super::states::ValkyrieConfig;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> ContractResult<Response> {
    // Execute
    let config = ValkyrieConfig {
        campaign_code_whitelist: vec![],
        boost_contract: None,
    };

    config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    boost_contract: Option<Addr>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, env, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    if let Some(boost_contract) = boost_contract {
        valkyrie_config.boost_contract = Some(boost_contract)
    }

    valkyrie_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn add_campaign_code_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    source_code_url: String,
    description: String,
    maintainer: Option<String>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, env, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let campaign_code = CampaignCode {
        code_id,
        source_code_url,
        description,
        maintainer,
    };
    campaign_code.save(deps.storage)?;

    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    if valkyrie_config.campaign_code_whitelist.contains(&code_id) {
        return Err(ContractError::Std(StdError::generic_err("Already whitelisted campaign code")))
    }

    valkyrie_config.campaign_code_whitelist.push(code_id);
    valkyrie_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn remove_campaign_code_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, env, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    let index = valkyrie_config.campaign_code_whitelist.iter()
        .position(|v| *v == code_id).unwrap();

    valkyrie_config.campaign_code_whitelist.remove(index);
    valkyrie_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}