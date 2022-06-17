use cosmwasm_std::{Binary, Decimal, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, SubMsg, to_binary, Uint128, coin};

use valkyrie::campaign_manager::execute_msgs::{CampaignInstantiateMsg, InstantiateMsg};
use valkyrie::common::{ContractResult, Denom};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::{find, make_response, validate_zero_to_one};

use crate::states::*;
use valkyrie::cw20::{query_cw20_balance, query_balance};
use cw20::Cw20ExecuteMsg;
use valkyrie::proxy::asset::AssetInfo;
use valkyrie::proxy::execute_msgs::{ExecuteMsg as ProxyExecuteMsg, SwapOperation};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    validate_zero_to_one(msg.add_pool_fee_rate, "add_pool_fee_rate")?;
    validate_zero_to_one(msg.add_pool_min_referral_reward_rate, "add_pool_min_referral_reward_rate")?;
    validate_zero_to_one(msg.remove_pool_fee_rate, "remove_pool_fee_rate")?;
    validate_zero_to_one(msg.fee_burn_ratio, "fee_burn_ratio")?;

    // Execute
    let response = make_response("instantiate");

    Config {
        governance: deps.api.addr_validate(msg.governance.as_str())?,
        vp_token: deps.api.addr_validate(msg.vp_token.as_str())?,
        valkyrie_token: deps.api.addr_validate(msg.valkyrie_token.as_str())?,
        valkyrie_proxy: deps.api.addr_validate(msg.valkyrie_proxy.as_str())?,
        code_id: msg.code_id,
        add_pool_fee_rate: msg.add_pool_fee_rate,
        add_pool_min_referral_reward_rate: msg.add_pool_min_referral_reward_rate,
        remove_pool_fee_rate: msg.remove_pool_fee_rate,
        fee_burn_ratio: msg.fee_burn_ratio,
        fee_recipient: deps.api.addr_validate(msg.fee_recipient.as_str())?,
        deactivate_period: msg.deactivate_period,
        key_denom: msg.key_denom.to_cw20(deps.api),
        contract_admin: deps.api.addr_validate(msg.contract_admin.as_str())?,
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
    valkyrie_token: Option<String>,
    vp_token: Option<String>,
    valkyrie_proxy: Option<String>,
    code_id: Option<u64>,
    add_pool_fee_rate: Option<Decimal>,
    add_pool_min_referral_reward_rate: Option<Decimal>,
    remove_pool_fee_rate: Option<Decimal>,
    fee_burn_ratio: Option<Decimal>,
    fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
    key_denom: Option<Denom>,
    contract_admin: Option<String>,
) -> ContractResult<Response> {
    // Validate
    let mut config = Config::load(deps.storage)?;

    if !config.is_governance(&info.sender) && !config.is_contract_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_contract_config");

    if let Some(valkyrie_token) = valkyrie_token.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.valkyrie_token = deps.api.addr_validate(valkyrie_token)?;
        response = response.add_attribute("is_updated_valkyrie_token", "true");
    }

    if let Some(vp_token) = vp_token.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.vp_token = deps.api.addr_validate(vp_token)?;
        response = response.add_attribute("is_updated_vp_token", "true");
    }

    if let Some(valkyrie_proxy) = valkyrie_proxy.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.valkyrie_proxy = deps.api.addr_validate(valkyrie_proxy)?;
        response = response.add_attribute("is_updated_valkyrie_proxy", "true");
    }

    if let Some(code_id) = code_id.as_ref() {
        config.code_id = *code_id;
        response = response.add_attribute("is_updated_code_id", "true");
    }

    if let Some(add_pool_fee_rate) = add_pool_fee_rate.as_ref() {
        validate_zero_to_one(*add_pool_fee_rate, "add_pool_fee_rate")?;

        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.add_pool_fee_rate = *add_pool_fee_rate;
        response = response.add_attribute("is_updated_add_pool_fee_rate", "true");
    }

    if let Some(add_pool_min_referral_reward_rate) = add_pool_min_referral_reward_rate.as_ref() {
        validate_zero_to_one(*add_pool_min_referral_reward_rate, "add_pool_min_referral_reward_rate")?;

        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.add_pool_min_referral_reward_rate = *add_pool_min_referral_reward_rate;
        response = response.add_attribute("is_updated_add_pool_min_referral_reward_rate", "true");
    }

    if let Some(remove_pool_fee_rate) = remove_pool_fee_rate.as_ref() {
        validate_zero_to_one(*remove_pool_fee_rate, "remove_pool_fee_rate")?;

        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.remove_pool_fee_rate = *remove_pool_fee_rate;
        response = response.add_attribute("is_updated_remove_pool_fee_rate", "true");
    }

    if let Some(fee_burn_ratio) = fee_burn_ratio.as_ref() {
        validate_zero_to_one(*fee_burn_ratio, "fee_burn_ratio")?;

        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.fee_burn_ratio = *fee_burn_ratio;
        response = response.add_attribute("is_updated_fee_burn_ratio", "true");
    }

    if let Some(fee_recipient) = fee_recipient.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.fee_recipient = deps.api.addr_validate(fee_recipient)?;
        response = response.add_attribute("is_updated_fee_recipient", "true");
    }

    if let Some(deactivate_period) = deactivate_period.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.deactivate_period = *deactivate_period;
        response = response.add_attribute("is_updated_deactivate_period", "true");
    }

    if let Some(key_denom) = key_denom.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.key_denom = key_denom.to_cw20(deps.api);
        response = response.add_attribute("is_updated_key_denom", "true");
    }

    if let Some(governance) = governance.as_ref() {
        if !config.is_governance(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        config.governance = deps.api.addr_validate(governance)?;
        response = response.add_attribute("is_updated_governance", "true");
    }

    if let Some(contract_admin) = contract_admin.as_ref() {
        Config::save_contract_admin_nominee(deps.storage, &deps.api.addr_validate(contract_admin)?)?;
        response = response.add_attribute("is_updated_contract_admin_nominee", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn approve_contract_admin_nominee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // Execute
    let mut response = make_response("approve_contract_admin_nominee");

    if let Some(admin_nominee) = Config::may_load_contract_admin_nominee(deps.storage)? {
        if admin_nominee != info.sender {
            return Err(ContractError::Std(StdError::generic_err("It is not contract admin nominee")));
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign_config = Config::load(deps.storage)?;
    campaign_config.contract_admin = info.sender;
    response = response.add_attribute("is_updated_contract_admin", "true");

    campaign_config.save(deps.storage)?;

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
        response = response.add_attribute("is_updated_overflow_amount_recipient", "true");
    }

    if let Some(base_count) = base_count.as_ref() {
        limit_option.base_count = *base_count;
        response = response.add_attribute("is_updated_base_count", "true");
    }

    if let Some(percent_for_governance_staking) = percent_for_governance_staking.as_ref() {
        limit_option.percent_for_governance_staking = *percent_for_governance_staking;
        response = response.add_attribute("is_updated_percent_for_governance_staking", "true");
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
    response = response.add_attribute("is_updated_overflow_amount_recipient", "true");

    limit_option.save(deps.storage)?;

    Ok(response)
}

pub const REPLY_CREATE_CAMPAIGN: u64 = 1;

pub fn create_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config_msg: Binary,
    deposit_denom: Option<Denom>,
    deposit_amount: Option<Uint128>,
    deposit_lock_period: Option<u64>,
    vp_burn_amount: Option<Uint128>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
) -> ContractResult<Response> {
    // Validate
    let config = Config::load(deps.storage)?;

    // Execute
    let mut response = make_response("create_campaign");

    CreateCampaignContext {
        code_id: config.code_id,
        creator: deps.api.addr_validate(info.sender.as_str())?,
    }.save(deps.storage)?;

    let create_campaign_msg = message_factories::wasm_instantiate(
        config.code_id,
        Some(config.contract_admin.clone()),
        to_binary(&CampaignInstantiateMsg {
            governance: config.governance.to_string(),
            campaign_manager: env.contract.address.to_string(),
            admin: info.sender.to_string(),
            creator: info.sender.to_string(),
            config_msg,
            deposit_denom,
            deposit_amount: deposit_amount.unwrap_or_default(),
            deposit_lock_period: deposit_lock_period.unwrap_or_default(),
            vp_token: config.vp_token.to_string(),
            vp_burn_amount: vp_burn_amount.unwrap_or_default(),
            qualifier,
            qualification_description,
            referral_reward_token: config.valkyrie_token.to_string(),
        })?,
    );

    response = response.add_submessage(SubMsg {
        id: REPLY_CREATE_CAMPAIGN,
        msg: create_campaign_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    response = response.add_attribute("campaign_code_id", config.code_id.to_string());
    response = response.add_attribute("campaign_creator", info.sender.to_string());
    response = response.add_attribute("campaign_admin", info.sender.to_string());

    Ok(response)
}

pub fn created_campaign(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> ContractResult<Response> {
    // Validate
    if msg.result.is_err() {
        return Err(ContractError::Std(StdError::generic_err(msg.result.unwrap_err())));
    }

    let events = msg.result.unwrap().events;
    let event = find(&events, |e| e.ty == "instantiate");
    if event.is_none() {
        return Err(ContractError::Std(StdError::generic_err("Failed to parse data")));
    }

    let contract_address = find(&event.unwrap().attributes, |a| a.key == "_contract_address");
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

    response = response.add_attribute("campaign_address", contract_address.to_string());

    Ok(response)
}

pub fn spend_fee(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let mut response = make_response("spend_fee");

    let config = Config::load(deps.storage)?;

    let amount = if let Some(amount) = amount {
        amount
    } else {
        query_cw20_balance(
            &deps.querier,
            &config.valkyrie_token,
            &env.contract.address,
        )?
    };

    let burn_amount = amount * config.fee_burn_ratio;
    let distribute_amount = amount.checked_sub(burn_amount)?;

    response = response.add_message(message_factories::wasm_execute(
        &config.valkyrie_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: config.fee_recipient.to_string(),
            amount: distribute_amount,
        },
    ));

    response = response.add_message(message_factories::wasm_execute(
        &config.valkyrie_token,
        &Cw20ExecuteMsg::Burn {
            amount: burn_amount,
        },
    ));

    Ok(response)
}

pub fn swap_fee(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
    route: Option<Vec<Denom>>,
) -> ContractResult<Response> {
    // Validate
    let config = Config::load(deps.storage)?;
    let token_denom = Denom::Token(config.valkyrie_token.to_string());
    let route = route.unwrap_or_else(|| vec![denom.clone(), token_denom.clone()]);

    if route.len() < 2 || *route.first().unwrap() != denom || *route.last().unwrap() != token_denom {
        return Err(ContractError::Std(StdError::generic_err(
            format!(
                "route must start with '{}' and end with '{}'",
                denom.to_string(), token_denom.to_string(),
            )
        )));
    }

    // Execute
    let mut response = make_response("swap_fee");

    let operations: Vec<SwapOperation> = route.windows(2).map(|pair| {
        pair_to_swap_operation(pair)
    }).collect();

    let swap_msg = ProxyExecuteMsg::ExecuteSwapOperations {
        operations,
        minimum_receive: None,
        to: None,
        max_spread: None,
    };

    let balance = query_balance(
        &deps.querier,
        denom.to_cw20(deps.api),
        env.contract.address.clone(),
    )?;
    let amount = if let Some(amount) = amount {
        if amount > balance {
            return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
        } else {
            amount
        }
    } else {
        balance
    };

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let swap_msg = match denom {
        Denom::Native(denom) => {
            message_factories::wasm_execute_with_funds(
                &config.valkyrie_proxy,
                vec![coin(amount.u128(), denom)],
                &swap_msg,
            )
        }
        Denom::Token(address) => {
            message_factories::wasm_execute(
                &deps.api.addr_validate(&address)?,
                &Cw20ExecuteMsg::Send {
                    contract: config.valkyrie_proxy.to_string(),
                    msg: to_binary(&swap_msg)?,
                    amount,
                },
            )
        }
    };

    response = response.add_message(swap_msg);

    Ok(response)
}

fn pair_to_swap_operation(pair: &[Denom]) -> SwapOperation {
    let left = pair[0].clone();
    let right = pair[1].clone();

    SwapOperation::Swap {
        offer_asset_info: denom_to_asset_info(left),
        ask_asset_info: denom_to_asset_info(right),
    }
}

fn denom_to_asset_info(denom: Denom) -> AssetInfo {
    match denom {
        Denom::Native(denom) => AssetInfo::NativeToken {
            denom,
        },
        Denom::Token(address) => AssetInfo::Token {
            contract_addr: address,
        },
    }
}
