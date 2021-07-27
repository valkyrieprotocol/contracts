use cosmwasm_std::{Binary, Decimal, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, SubMsg, to_binary, Uint128};

use valkyrie::campaign::execute_msgs::ExecuteMsg as CampaignExecuteMsg;
use valkyrie::campaign::query_msgs::{ActiveBoosterResponse, CampaignStateResponse, QueryMsg};
use valkyrie::campaign_manager::execute_msgs::{CampaignInstantiateMsg, InstantiateMsg};
use valkyrie::common::{ContractResult, Denom, ExecutionMsg};
use valkyrie::errors::ContractError;
use valkyrie::fund_manager::execute_msgs::ExecuteMsg;
use valkyrie::message_factories;
use valkyrie::utils::{find, make_response};

use crate::states::{BoosterConfig, Campaign, CampaignConfig, ContractConfig, CreateCampaignContext, is_governance};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

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

    Ok(response)
}

pub fn update_contract_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    governance: Option<String>,
    fund_manager: Option<String>,
) -> ContractResult<Response> {
    // Validate
    let mut config = ContractConfig::load(deps.storage)?;

    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_contract_config");

    if let Some(governance) = governance.as_ref() {
        config.governance = deps.api.addr_validate(governance)?;
        response.add_attribute("is_updated_governance", "true");
    }

    if let Some(fund_manager) = fund_manager.as_ref() {
        config.fund_manager = deps.api.addr_validate(fund_manager)?;
        response.add_attribute("is_updated_fund_manager", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
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
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_campaign_config");

    let mut config = CampaignConfig::load(deps.storage)?;

    if let Some(creation_fee_token) = creation_fee_token.as_ref() {
        config.creation_fee_token = deps.api.addr_validate(creation_fee_token)?;
        response.add_attribute("is_updated_creation_fee_token", "true");
    }

    if let Some(creation_fee_amount) = creation_fee_amount.as_ref() {
        config.creation_fee_amount = *creation_fee_amount;
        response.add_attribute("is_updated_creation_fee_amount", "true");
    }

    if let Some(creation_fee_recipient) = creation_fee_recipient.as_ref() {
        config.creation_fee_recipient = deps.api.addr_validate(creation_fee_recipient)?;
        response.add_attribute("is_updated_creation_fee_recipient", "true");
    }

    if let Some(code_id) = code_id.as_ref() {
        config.code_id = *code_id;
        response.add_attribute("is_updated_code_id", "true");
    }

    if let Some(withdraw_fee_rate) = withdraw_fee_rate.as_ref() {
        config.withdraw_fee_rate = *withdraw_fee_rate;
        response.add_attribute("is_updated_withdraw_fee_rate", "true");
    }

    if let Some(withdraw_fee_recipient) = withdraw_fee_recipient.as_ref() {
        config.withdraw_fee_recipient = deps.api.addr_validate(withdraw_fee_recipient)?;
        response.add_attribute("is_updated_withdraw_fee_recipient", "true");
    }

    if let Some(deactivate_period) = deactivate_period.as_ref() {
        config.deactivate_period = *deactivate_period;
        response.add_attribute("is_updated_deactivate_period", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
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
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_booster_config");

    let mut config = BoosterConfig::load(deps.storage)?;

    if let Some(booster_token) = booster_token.as_ref() {
        config.booster_token = deps.api.addr_validate(booster_token)?;
        response.add_attribute("is_updated_booster_token", "true");
    }

    if let Some(drop_ratio) = drop_ratio.as_ref() {
        config.drop_ratio = *drop_ratio;
        response.add_attribute("is_updated_drop_ratio", "true");
    }

    if let Some(activity_ratio) = activity_ratio.as_ref() {
        config.activity_ratio = *activity_ratio;
        response.add_attribute("is_updated_activity_ratio", "true");
    }

    if let Some(plus_ratio) = plus_ratio.as_ref() {
        config.plus_ratio = *plus_ratio;
        response.add_attribute("is_updated_plus_ratio", "true");
    }

    if let Some(activity_multiplier) = activity_multiplier.as_ref() {
        config.activity_multiplier = *activity_multiplier;
        response.add_attribute("is_updated_activity_multiplier", "true");
    }

    if let Some(min_participation_count) = min_participation_count.as_ref() {
        config.min_participation_count = *min_participation_count;
        response.add_attribute("is_updated_min_participation_count", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn add_distribution_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: Denom,
) -> ContractResult<Response> {
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    let mut config = CampaignConfig::load(deps.storage)?;

    let denom = denom.to_cw20(deps.api);
    if config.distribution_denom_whitelist.contains(&denom) {
        return Err(ContractError::AlreadyExists {});
    }

    // Execute
    let mut response = make_response("add_distribution_denom");

    config.distribution_denom_whitelist.push(denom);

    config.save(deps.storage)?;

    response.add_attribute("whitelist_size", config.distribution_denom_whitelist.len().to_string());

    Ok(response)
}

pub fn remove_distribution_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: Denom,
) -> ContractResult<Response> {
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("remove_distribution_denom");

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

    response.add_attribute("whitelist_size", config.distribution_denom_whitelist.len().to_string());

    Ok(response)
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
        return Err(ContractError::Std(StdError::generic_err("Invalid creation fee token")));
    }

    if amount < campaign_config.creation_fee_amount {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Insufficient creation fee (Fee = {})", campaign_config.creation_fee_amount),
        )));
    }

    let contract_config = ContractConfig::load(deps.storage)?;

    // Execute
    let mut response = make_response("create_campaign");

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

    response.add_submessage(SubMsg {
        id: REPLY_CREATE_CAMPAIGN,
        msg: create_campaign_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    if !amount.is_zero() {
        response.add_message(message_factories::cw20_transfer(
            &campaign_config.creation_fee_token,
            &campaign_config.creation_fee_recipient,
            amount,
        ));
    }

    response.add_attribute("campaign_code_id", campaign_config.code_id.to_string());
    response.add_attribute("campaign_creator", sender.clone());
    response.add_attribute("campaign_admin", sender.clone());

    Ok(response)
}

pub fn created_campaign(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> ContractResult<Response> {
    // Validate
    let events = msg.result.unwrap().events;
    let event = find(&events, |e| e.ty == "instantiate_contract");
    if event.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Failed to parse data")));
    }

    let contract_address = find(&event.unwrap().attributes, |a| a.key == "contract_address");
    if contract_address.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Failed to parse data")));
    }
    let contract_address = &contract_address.unwrap().value;

    // Execute
    let mut response = make_response("created_campaign");
    let context = CreateCampaignContext::load(deps.storage)?;

    Campaign {
        code_id: context.code_id,
        address: deps.api.addr_validate(contract_address)?,
        creator: context.creator,
        created_height: env.block.height,
    }.save(deps.storage)?;

    CreateCampaignContext::clear(deps.storage);

    response.add_attribute("campaign_address", contract_address.to_string());

    Ok(response)
}

pub fn boost_campaign(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    let campaign = deps.api.addr_validate(&campaign)?;
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let contract_config = ContractConfig::load(deps.storage)?;
    let booster_config = BoosterConfig::load(deps.storage)?;

    let campaign_state: CampaignStateResponse = deps.querier.query_wasm_smart(
        campaign.to_string(),
        &QueryMsg::CampaignState {},
    )?;

    if campaign_state.participation_count < booster_config.min_participation_count {
        return Err(ContractError::Std(StdError::generic_err("Not satisfied min_participation_count")));
    }

    if campaign_state.is_pending {
        return Err(ContractError::Std(StdError::generic_err("Can not boost in pending state")));
    }

    // Execute
    let mut response = make_response("boost_campaign");

    let drop_booster_amount = booster_config.drop_ratio * amount;
    let activity_booster_amount = booster_config.activity_ratio * amount;
    let plus_booster_amount = booster_config.plus_ratio * amount;

    response.add_message(message_factories::wasm_execute(
        &contract_config.fund_manager,
        &ExecuteMsg::IncreaseAllowance {
            address: campaign.to_string(),
            amount,
        },
    ));

    response.add_message(message_factories::wasm_execute(
        &campaign,
        &CampaignExecuteMsg::EnableBooster {
            drop_booster_amount: drop_booster_amount.clone(),
            activity_booster_amount: activity_booster_amount.clone(),
            plus_booster_amount: plus_booster_amount.clone(),
            activity_booster_multiplier: booster_config.activity_multiplier.clone(),
        },
    ));

    response.add_attribute("participation_count", campaign_state.participation_count.to_string());
    response.add_attribute("drop_booster_amount", drop_booster_amount.to_string());
    response.add_attribute("activity_booster_amount", activity_booster_amount.to_string());
    response.add_attribute("plus_booster_amount", plus_booster_amount.to_string());
    response.add_attribute("activity_booster_multiplier", booster_config.activity_multiplier.to_string());

    Ok(response)
}

pub fn finish_boosting(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign: String,
) -> ContractResult<Response> {
    // Validate
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

    // Execute
    let mut response = make_response("finish_boosting");

    let drop_booster_left_amount = booster_state.drop_booster.assigned_amount
        .checked_sub(booster_state.drop_booster.calculated_amount)?;
    let activity_booster_left_amount = booster_state.activity_booster.assigned_amount
        .checked_sub(booster_state.activity_booster.distributed_amount)?;
    let plus_booster_left_amount = booster_state.plus_booster.assigned_amount
        .checked_sub(booster_state.plus_booster.distributed_amount)?;

    let release_amount = drop_booster_left_amount + activity_booster_left_amount + plus_booster_left_amount;

    if !release_amount.is_zero() {
        let contract_config = ContractConfig::load(deps.storage)?;

        response.add_message(message_factories::wasm_execute(
            &contract_config.fund_manager,
            &ExecuteMsg::DecreaseAllowance {
                address: campaign.to_string(),
                amount: Some(release_amount),
            },
        ));
    }

    response.add_message(message_factories::wasm_execute(
        &campaign,
        &CampaignExecuteMsg::DisableBooster {},
    ));

    response.add_attribute("drop_booster_left_amount", drop_booster_left_amount.to_string());
    response.add_attribute("activity_booster_left_amount", activity_booster_left_amount.to_string());
    response.add_attribute("plus_booster_left_amount", plus_booster_left_amount.to_string());

    Ok(response)
}

