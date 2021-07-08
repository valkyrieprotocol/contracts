use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Decimal, Uint128, Binary, StdError, to_binary, SubMsg, ReplyOn, Reply, CosmosMsg, WasmMsg};
use valkyrie::campaign_manager::execute_msgs::{InstantiateMsg, CampaignInstantiateMsg};
use valkyrie::common::{ContractResult, Denom, ExecutionMsg};
use crate::states::{ContractConfig, CampaignConfig, BoosterConfig, is_governance, CreateCampaignContext, Campaign};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::find;
use valkyrie::fund_manager::execute_msgs::ExecuteMsg::{IncreaseAllowance, DecreaseAllowance};
use valkyrie::campaign::query_msgs::{QueryMsg, CampaignStateResponse, ActiveBoosterResponse};
use valkyrie::campaign::execute_msgs::ExecuteMsg as CampaignExecuteMsg;
use std::ops::Add;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    ContractConfig {
        governance: deps.api.addr_validate(msg.contract_config.governance.as_str())?,
        fund_manager: deps.api.addr_validate(msg.contract_config.fund_manager.as_str())?,
    }.save(deps.storage)?;

    CampaignConfig {
        creation_fee_token: deps.api.addr_validate(msg.campaign_config.creation_fee_token.as_str())?,
        creation_fee_amount: msg.campaign_config.creation_fee_amount,
        creation_fee_recipient: deps.api.addr_validate(msg.campaign_config.creation_fee_recipient.as_str())?,
        code_id: msg.campaign_config.code_id,
        distribution_denom_whitelist: msg.campaign_config.distribution_denom_whitelist.iter().map(|d| d.to_cw20(deps.api)).collect(),
        withdraw_fee_rate: msg.campaign_config.withdraw_fee_rate,
        withdraw_fee_recipient: deps.api.addr_validate(msg.campaign_config.withdraw_fee_recipient.as_str())?,
        deactivate_period: msg.campaign_config.deactivate_period,
    }.save(deps.storage)?;

    BoosterConfig {
        booster_token: deps.api.addr_validate(msg.booster_config.booster_token.as_str())?,
        drop_ratio: msg.booster_config.drop_booster_ratio,
        activity_ratio: msg.booster_config.activity_booster_ratio,
        plus_ratio: msg.booster_config.plus_booster_ratio,
        activity_multiplier: msg.booster_config.activity_booster_multiplier,
        min_participation_count: msg.booster_config.min_participation_count,
    }.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_contract_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    governance: Option<String>,
    fund_manager: Option<String>,
) -> ContractResult<Response> {
    let mut config = ContractConfig::load(deps.storage)?;

    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(governance) = governance {
        config.governance = deps.api.addr_validate(governance.as_str())?;
    }

    if let Some(fund_manager) = fund_manager {
        config.fund_manager = deps.api.addr_validate(fund_manager.as_str())?;
    }

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_campaign_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    creation_fee_token: Option<String>,
    creation_fee_amount: Option<Uint128>,
    creation_fee_recipient: Option<String>,
    code_id: Option<u64>,
    withdraw_fee_rate: Option<Decimal>,
    withdraw_fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
) -> ContractResult<Response> {
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CampaignConfig::load(deps.storage)?;

    if let Some(creation_fee_token) = creation_fee_token {
        config.creation_fee_token = deps.api.addr_validate(creation_fee_token.as_str())?;
    }

    if let Some(creation_fee_amount) = creation_fee_amount {
        config.creation_fee_amount = creation_fee_amount;
    }

    if let Some(creation_fee_recipient) = creation_fee_recipient {
        config.creation_fee_recipient = deps.api.addr_validate(creation_fee_recipient.as_str())?;
    }

    if let Some(code_id) = code_id {
        config.code_id = code_id;
    }

    if let Some(withdraw_fee_rate) = withdraw_fee_rate {
        config.withdraw_fee_rate = withdraw_fee_rate;
    }

    if let Some(withdraw_fee_recipient) = withdraw_fee_recipient {
        config.withdraw_fee_recipient = deps.api.addr_validate(withdraw_fee_recipient.as_str())?;
    }

    if let Some(deactivate_period) = deactivate_period {
        config.deactivate_period = deactivate_period;
    }

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_booster_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    booster_token: Option<String>,
    drop_ratio: Option<Decimal>,
    activity_ratio: Option<Decimal>,
    plus_ratio: Option<Decimal>,
    activity_multiplier: Option<Decimal>,
    min_participation_count: Option<u64>,
) -> ContractResult<Response> {
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = BoosterConfig::load(deps.storage)?;

    if let Some(booster_token) = booster_token {
        config.booster_token = deps.api.addr_validate(booster_token.as_str())?;
    }

    if let Some(drop_ratio) = drop_ratio {
        config.drop_ratio = drop_ratio;
    }

    if let Some(activity_ratio) = activity_ratio {
        config.activity_ratio = activity_ratio;
    }

    if let Some(plus_ratio) = plus_ratio {
        config.plus_ratio = plus_ratio;
    }

    if let Some(activity_multiplier) = activity_multiplier {
        config.activity_multiplier = activity_multiplier;
    }

    if let Some(min_participation_count) = min_participation_count {
        config.min_participation_count = min_participation_count;
    }

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn add_distribution_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: Denom,
) -> ContractResult<Response> {
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CampaignConfig::load(deps.storage)?;

    config.distribution_denom_whitelist.push(denom.to_cw20(deps.api));

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub fn remove_distribution_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: Denom,
) -> ContractResult<Response> {
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CampaignConfig::load(deps.storage)?;

    let denom = denom.to_cw20(deps.api);
    let position = config.distribution_denom_whitelist.iter()
        .position(|d| denom == *d);

    if let Some(position) = position {
        config.distribution_denom_whitelist.remove(position);
    } else {
        return Err(ContractError::NotFound {});
    }

    config.save(deps.storage)?;

    Ok(Response::default())
}

pub const REPLY_CREATE_CAMPAIGN: u64 = 1;

pub fn create_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
    config_msg: Binary,
    proxies: Vec<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    // Validate
    let campaign_config = CampaignConfig::load(deps.storage)?;

    if info.sender != campaign_config.creation_fee_token {
        return Err(ContractError::Std(StdError::generic_err("Invalid creation fee denom")));
    }

    if amount < campaign_config.creation_fee_amount {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Insufficient creation fee (Fee = {})", campaign_config.creation_fee_amount),
        )));
    }

    let contract_config = ContractConfig::load(deps.storage)?;

    // Execute
    CreateCampaignContext {
        code_id: campaign_config.code_id,
        creator: deps.api.addr_validate(sender.as_str())?,
    }.save(deps.storage)?;

    let create_campaign_msg = message_factories::wasm_instantiate(
        campaign_config.code_id,
        Some(contract_config.governance.clone()),
        to_binary(&CampaignInstantiateMsg {
            governance: contract_config.governance.to_string(),
            campaign_manager: env.contract.address.to_string(),
            fund_manager: contract_config.fund_manager.to_string(),
            admin: sender.to_string(),
            creator: sender.to_string(),
            proxies,
            config_msg,
            executions,
        })?,
    );

    let fee_send_msg = message_factories::cw20_transfer(
        &campaign_config.creation_fee_token,
        &campaign_config.creation_fee_recipient,
        amount,
    );

    // Response
    Ok(Response {
        submessages: vec![
            SubMsg {
                id: REPLY_CREATE_CAMPAIGN,
                msg: create_campaign_msg,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            }
        ],
        messages: vec![fee_send_msg],
        attributes: vec![],
        data: None,
    })
}

pub fn created_campaign(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> ContractResult<Response> {
    let events = msg.result.unwrap().events;
    let event = find(&events, |e| e.kind == "instantiate_contract");
    if event.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Failed to parse data")));
    }

    let contract_address = find(&event.unwrap().attributes, |a| a.key == "contract_address");
    if contract_address.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Failed to parse data")));
    }
    let contract_address = &contract_address.unwrap().value;

    let context = CreateCampaignContext::load(deps.storage)?;

    Campaign {
        code_id: context.code_id,
        address: deps.api.addr_validate(contract_address)?,
        creator: context.creator,
        created_height: env.block.height,
    }.save(deps.storage)?;

    CreateCampaignContext::clear(deps.storage);

    Ok(Response::default())
}

pub fn boost_campaign(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign: String,
    amount: Uint128,
) -> ContractResult<Response> {
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let contract_config = ContractConfig::load(deps.storage)?;
    let booster_config = BoosterConfig::load(deps.storage)?;

    let campaign_state: CampaignStateResponse = deps.querier.query_wasm_smart(
        campaign.clone(),
        &QueryMsg::CampaignState {},
    )?;

    if campaign_state.participation_count < booster_config.min_participation_count {
        return Err(ContractError::Std(StdError::generic_err("Not satisfied min_participation_count")));
    }

    if campaign_state.is_pending {
        return Err(ContractError::Std(StdError::generic_err("Can not boost in pending state")));
    }

    let drop_booster_amount = booster_config.drop_ratio * amount;
    let activity_booster_amount = booster_config.activity_ratio * amount;
    let plus_booster_amount = booster_config.plus_ratio * amount;

    let allowance_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_config.fund_manager.to_string(),
        send: vec![],
        msg: to_binary(&IncreaseAllowance {
            address: campaign.clone(),
            amount,
        })?,
    });

    let enable_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: campaign,
        send: vec![],
        msg: to_binary(&CampaignExecuteMsg::EnableBooster {
            drop_booster_amount,
            activity_booster_amount,
            plus_booster_amount,
            activity_booster_multiplier: booster_config.activity_multiplier,
        })?,
    });

    Ok(Response {
        submessages: vec![],
        messages: vec![allowance_msg, enable_msg],
        attributes: vec![],
        data: None,
    })
}

pub fn finish_boosting(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign: String,
) -> ContractResult<Response> {
    let campaign = deps.api.addr_validate(campaign.as_str())?;
    if !is_governance(deps.storage, &info.sender) && campaign != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let booster_state: ActiveBoosterResponse = deps.querier.query_wasm_smart(
        &campaign,
        &QueryMsg::ActiveBooster {},
    )?;
    if booster_state.active_booster.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Not exist active booster")));
    }
    let booster_state = booster_state.active_booster.unwrap();

    let release_amount = booster_state.drop_booster.assigned_amount
        .checked_sub(booster_state.drop_booster.calculated_amount)?
        .add(booster_state.activity_booster.assigned_amount)
        .checked_sub(booster_state.activity_booster.distributed_amount)?
        .add(booster_state.plus_booster.assigned_amount)
        .checked_sub(booster_state.plus_booster.distributed_amount)?;

    let contract_config = ContractConfig::load(deps.storage)?;

    let revoke_allowance_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_config.fund_manager.to_string(),
        send: vec![],
        msg: to_binary(&DecreaseAllowance {
            address: campaign.to_string(),
            amount: Some(release_amount),
        })?,
    });

    let disable_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: campaign.to_string(),
        send: vec![],
        msg: to_binary(&CampaignExecuteMsg::DisableBooster {})?,
    });

    Ok(Response {
        submessages: vec![],
        messages: vec![revoke_allowance_msg, disable_msg],
        attributes: vec![],
        data: None,
    })
}

