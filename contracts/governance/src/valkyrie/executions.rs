use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::common::state::ContractConfig;
use crate::valkyrie::state::CampaignCode;

use super::state::ValkyrieConfig;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> ContractResult<Response> {
    let config = ValkyrieConfig {
        campaign_code_whitelist: vec![],
        boost_contract: None,
    };

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    boost_contract: Option<Addr>,
) -> ContractResult<Response> {
    let contract_config = ContractConfig::load(deps.storage)?;

    if !contract_config.is_admin(deps.api.addr_canonicalize(info.sender.as_str())?) {
        return Err(ContractError::Unauthorized {});
    }

    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    if let Some(boost_contract) = boost_contract {
        valkyrie_config.boost_contract = Some(deps.api.addr_canonicalize(boost_contract.as_str())?)
    }

    valkyrie_config.save(deps.storage)?;

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
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let campaign_code = CampaignCode {
        code_id,
        source_code_url,
        description,
        maintainer,
    };
    campaign_code.save(deps.storage)?;

    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    valkyrie_config.campaign_code_whitelist.push(code_id);
    valkyrie_config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn remove_campaign_code_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
) -> ContractResult<Response> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    let index = valkyrie_config.campaign_code_whitelist.iter()
        .position(|v| *v == code_id).unwrap();

    valkyrie_config.campaign_code_whitelist.remove(index);
    valkyrie_config.save(deps.storage)?;

    Ok(Response::default())
}