use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use valkyrie::campaign::execute_msgs::ExecuteMsg as CampaignExecuteMsg;
use valkyrie::common::ContractResult;
use valkyrie::distributor::execute_msgs::BoosterConfig;
use valkyrie::errors::ContractError;

use crate::states::{CampaignInfo, ContractConfig};

pub fn update_booster_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    booster_config: BoosterConfig,
) -> ContractResult<Response> {
    let mut config = ContractConfig::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    config.booster_config = booster_config.clone();
    config.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "update_booster_config"),
            attr("drop_booster_ratio", booster_config.drop_booster_ratio),
            attr(
                "activity_booster_ratio",
                booster_config.activity_booster_ratio,
            ),
            attr("plus_booster_ratio", booster_config.plus_booster_ratio),
        ],
        data: None,
    })
}

pub fn add_campaign(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign_addr: String,
    spend_limit: Uint128,
) -> ContractResult<Response> {
    // only governance contract can execute this message
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let campaign_addr = deps.api.addr_validate(&campaign_addr)?;
    if CampaignInfo::load(deps.storage, &campaign_addr).is_ok() {
        return Err(ContractError::AlreadyExists {});
    }

    let campaign_info = CampaignInfo {
        campaign: campaign_addr.clone(),
        spend_limit,
    };

    campaign_info.save(deps.storage)?;

    let drop_booster_amount = config.booster_config.drop_booster_ratio * spend_limit;
    let activity_booster_amount = config.booster_config.activity_booster_ratio * spend_limit;
    let plus_booster_amount = config.booster_config.plus_booster_ratio * spend_limit;

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: campaign_addr.to_string(),
            send: vec![],
            msg: to_binary(&CampaignExecuteMsg::RegisterBooster {
                drop_booster_amount,
                activity_booster_amount,
                plus_booster_amount,
            })?,
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "add_campaign"),
            attr("campaign_addr", campaign_addr),
            attr("spend_limit", spend_limit),
            attr("drop_booster_amount", drop_booster_amount),
            attr("activity_booster_amount", activity_booster_amount),
            attr("plus_booster_amount", plus_booster_amount),
        ],
        data: None,
    })
}

pub fn remove_campaign(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign_addr: String,
) -> ContractResult<Response> {
    // only governance contract can execute this message
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Ok(campaign_info) =
        CampaignInfo::load(deps.storage, &deps.api.addr_validate(&campaign_addr)?)
    {
        campaign_info.remove(deps.storage);
    } else {
        return Err(ContractError::NotFound {});
    }

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: campaign_addr.clone(),
            send: vec![],
            msg: to_binary(&CampaignExecuteMsg::DeregisterBooster {})?,
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "remove_campaign"),
            attr("campaign_addr", campaign_addr),
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
    let mut campaign_info: CampaignInfo = CampaignInfo::load(deps.storage, &info.sender)?;
    if campaign_info.spend_limit < amount {
        return Err(ContractError::ExceedLimit {});
    }

    campaign_info.spend_limit = campaign_info.spend_limit.checked_sub(amount)?;
    if campaign_info.spend_limit.is_zero() {
        campaign_info.remove(deps.storage);
    } else {
        campaign_info.save(deps.storage)?;
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
            attr("campaign_addr", info.sender.as_str()),
            attr("recipient", recipient),
            attr("amount", amount),
        ],
        data: None,
    })
}
