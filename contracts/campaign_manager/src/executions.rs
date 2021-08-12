use cosmwasm_std::{Binary, Decimal, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, SubMsg, to_binary, Uint128};

use valkyrie::campaign_manager::execute_msgs::{CampaignInstantiateMsg, InstantiateMsg};
use valkyrie::common::{ContractResult, ExecutionMsg, Denom};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::{find, make_response};

use crate::states::*;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    Config {
        governance: deps.api.addr_validate(msg.governance.as_str())?,
        fund_manager: deps.api.addr_validate(msg.fund_manager.as_str())?,
        terraswap_router: deps.api.addr_validate(msg.terraswap_router.as_str())?,
        creation_fee_token: deps.api.addr_validate(msg.creation_fee_token.as_str())?,
        creation_fee_amount: msg.creation_fee_amount,
        creation_fee_recipient: deps.api.addr_validate(msg.creation_fee_recipient.as_str())?,
        code_id: msg.code_id,
        withdraw_fee_rate: msg.withdraw_fee_rate,
        withdraw_fee_recipient: deps.api.addr_validate(msg.withdraw_fee_recipient.as_str())?,
        deactivate_period: msg.deactivate_period,
        key_denom: msg.key_denom.to_cw20(deps.api),
        referral_reward_token: deps.api.addr_validate(msg.referral_reward_token.as_str())?,
        min_referral_reward_deposit_rate: msg.min_referral_reward_deposit_rate,
    }.save(deps.storage)?;

    ReferralRewardLimitOption {
        overflow_amount_recipient: msg.referral_reward_limit_option.overflow_amount_recipient
            .map(|r| deps.api.addr_validate(r.as_str()).unwrap()),
        base_count: msg.referral_reward_limit_option.base_count,
        percent_for_governance_staking: msg.referral_reward_limit_option.percent_for_governance_staking,
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    governance: Option<String>,
    fund_manager: Option<String>,
    terraswap_router: Option<String>,
    creation_fee_token: Option<String>,
    creation_fee_amount: Option<Uint128>,
    creation_fee_recipient: Option<String>,
    code_id: Option<u64>,
    withdraw_fee_rate: Option<Decimal>,
    withdraw_fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
    key_denom: Option<Denom>,
    referral_reward_token: Option<String>,
    min_referral_reward_deposit_rate: Option<Decimal>,
) -> ContractResult<Response> {
    // Validate
    let mut config = Config::load(deps.storage)?;

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

    if let Some(terraswap_router) = terraswap_router.as_ref() {
        config.terraswap_router = deps.api.addr_validate(terraswap_router)?;
        response.add_attribute("is_updated_terraswap_router", "true");
    }

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

    if let Some(key_denom) = key_denom.as_ref() {
        config.key_denom = key_denom.to_cw20(deps.api);
        response.add_attribute("is_updated_key_denom", "true");
    }

    if let Some(referral_reward_token) = referral_reward_token.as_ref() {
        config.referral_reward_token = deps.api.addr_validate(referral_reward_token)?;
        response.add_attribute("is_updated_referral_reward_token", "true");
    }

    if let Some(min_referral_reward_deposit_rate) = min_referral_reward_deposit_rate.as_ref() {
        config.min_referral_reward_deposit_rate = *min_referral_reward_deposit_rate;
        response.add_attribute("is_updated_min_referral_reward_deposit_rate", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn update_referral_reward_limit_option(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    overflow_amount_recipient: Option<String>,
    base_count: Option<u8>,
    percent_for_governance_staking: Option<u16>,
) -> ContractResult<Response> {
    // Validate
    let config = Config::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_referral_reward_limit_option");

    let mut limit_option = ReferralRewardLimitOption::load(deps.storage)?;

    if let Some(overflow_amount_recipient) = overflow_amount_recipient.as_ref() {
        limit_option.overflow_amount_recipient = Some(deps.api.addr_validate(overflow_amount_recipient.as_str())?);
        response.add_attribute("is_updated_overflow_amount_recipient", "true");
    }

    if let Some(base_count) = base_count.as_ref() {
        limit_option.base_count = *base_count;
        response.add_attribute("is_updated_base_count", "true");
    }

    if let Some(percent_for_governance_staking) = percent_for_governance_staking.as_ref() {
        limit_option.percent_for_governance_staking = *percent_for_governance_staking;
        response.add_attribute("is_updated_percent_for_governance_staking", "true");
    }

    limit_option.save(deps.storage)?;

    Ok(response)
}

pub fn set_reuse_overflow_amount(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // Validate
    let config = Config::load(deps.storage)?;
    if !config.is_governance(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("set_reuse_overflow_amount");

    let mut limit_option = ReferralRewardLimitOption::load(deps.storage)?;

    limit_option.overflow_amount_recipient = None;
    response.add_attribute("is_updated_overflow_amount_recipient", "true");

    limit_option.save(deps.storage)?;

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
    qualifier: Option<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    // Validate
    let config = Config::load(deps.storage)?;

    if info.sender != config.creation_fee_token {
        return Err(ContractError::Std(StdError::generic_err("Invalid creation fee token")));
    }

    if amount < config.creation_fee_amount {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Insufficient creation fee (Fee = {})", config.creation_fee_amount),
        )));
    }

    // Execute
    let mut response = make_response("create_campaign");

    CreateCampaignContext {
        code_id: config.code_id,
        creator: deps.api.addr_validate(sender.as_str())?,
    }.save(deps.storage)?;

    let create_campaign_msg = message_factories::wasm_instantiate(
        config.code_id,
        Some(config.governance.clone()),
        to_binary(&CampaignInstantiateMsg {
            governance: config.governance.to_string(),
            fund_manager: config.fund_manager.to_string(),
            campaign_manager: env.contract.address.to_string(),
            admin: sender.to_string(),
            creator: sender.to_string(),
            config_msg,
            qualifier,
            executions,
            referral_reward_token: config.referral_reward_token.to_string(),
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
            &config.creation_fee_token,
            &config.creation_fee_recipient,
            amount,
        ));
    }

    response.add_attribute("campaign_code_id", config.code_id.to_string());
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
