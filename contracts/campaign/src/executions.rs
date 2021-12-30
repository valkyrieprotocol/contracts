use cosmwasm_std::{Addr, Api, attr, Binary, CosmosMsg, Decimal, DepsMut, Env, from_binary, MessageInfo, QuerierWrapper, Reply, ReplyOn, Response, StdError, StdResult, Storage, SubMsg, to_binary, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Denom as Cw20Denom};
use protobuf::Message;
use terraswap::asset::AssetInfo;
use terraswap::router::{QueryMsg, SimulateSwapOperationsResponse, SwapOperation};

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::execute_msgs::{CampaignConfigMsg, DistributeResult, MigrateMsg, ReferralReward};
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::campaign_manager::query_msgs::ReferralRewardLimitOptionResponse;
use valkyrie::common::{ContractResult, Denom};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::{calc_ratio_amount, make_response};
use valkyrie_qualifier::{QualificationMsg, QualificationResult};
use valkyrie_qualifier::execute_msgs::ExecuteMsg as QualifierExecuteMsg;

use crate::proto::MsgExecuteContractResponse;
use crate::states::*;

pub const MIN_TITLE_LENGTH: usize = 4;
pub const MAX_TITLE_LENGTH: usize = 64;
pub const MIN_DESC_LENGTH: usize = 4;
pub const MAX_DESC_LENGTH: usize = 1024;
pub const MIN_URL_LENGTH: usize = 12;
pub const MAX_URL_LENGTH: usize = 128;
pub const MIN_PARAM_KEY_LENGTH: usize = 1;
pub const MAX_PARAM_KEY_LENGTH: usize = 16;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: CampaignInstantiateMsg,
) -> ContractResult<Response> {
    // Validate
    let campaign_config: CampaignConfigMsg = from_binary(&msg.config_msg)?;

    validate_title(&campaign_config.title)?;
    validate_url(&campaign_config.url)?;
    validate_description(&campaign_config.description)?;
    validate_parameter_key(&campaign_config.parameter_key)?;

    if let Some(desc) = msg.qualification_description.as_ref() {
        validate_qualification_description(desc)?;
    }

    // Execute
    let response = make_response("instantiate");

    CampaignConfig {
        governance: deps.api.addr_validate(&msg.governance)?,
        campaign_manager: deps.api.addr_validate(&msg.campaign_manager)?,
        title: campaign_config.title,
        description: campaign_config.description,
        url: campaign_config.url,
        parameter_key: campaign_config.parameter_key,
        deposit_denom: msg.deposit_denom.map(|d| d.to_cw20(deps.api)),
        deposit_amount: msg.deposit_amount,
        deposit_lock_period: msg.deposit_lock_period,
        qualifier: msg.qualifier.map(|q| deps.api.addr_validate(q.as_str())).transpose()?,
        qualification_description: msg.qualification_description,
        admin: deps.api.addr_validate(&msg.admin)?,
        creator: deps.api.addr_validate(&msg.creator)?,
        created_at: env.block.time,
    }.save(deps.storage)?;

    CampaignState::new(env.block.chain_id).save(deps.storage)?;

    RewardConfig {
        participation_reward_denom: campaign_config.participation_reward_denom.to_cw20(deps.api),
        participation_reward_amount: campaign_config.participation_reward_amount,
        participation_reward_lock_period: campaign_config.participation_reward_lock_period,
        referral_reward_token: deps.api.addr_validate(msg.referral_reward_token.as_str())?,
        referral_reward_amounts: campaign_config.referral_reward_amounts,
        referral_reward_lock_period: campaign_config.referral_reward_lock_period,
    }.save(deps.storage)?;

    Ok(response)
}

pub fn migrate(
    deps: DepsMut,
    env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response> {
    // Execute
    let mut response = make_response("migrate");
    response = response.add_attribute("chain_id", env.block.chain_id.clone());

    let mut campaign_state = CampaignState::load(deps.storage)?;

    campaign_state.chain_id = env.block.chain_id;

    campaign_state.save(deps.storage)?;

    Ok(response)
}

pub fn update_campaign_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    deposit_amount: Option<Uint128>,
    deposit_lock_period: Option<u64>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    admin: Option<String>,
) -> ContractResult<Response> {
    // Validate
    let mut campaign_config = CampaignConfig::load(deps.storage)?;
    if !campaign_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_campaign_config");

    if let Some(title) = title.as_ref() {
        validate_title(title)?;
        campaign_config.title = title.clone();
        response = response.add_attribute("is_updated_title", "true");
    }

    if let Some(description) = description.as_ref() {
        validate_description(description)?;
        campaign_config.description = description.clone();
        response = response.add_attribute("is_updated_description", "true");
    }

    if let Some(url) = url.as_ref() {
        validate_url(url)?;

        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_config.url = url.clone();
        response = response.add_attribute("is_updated_url", "true");
    }

    if let Some(parameter_key) = parameter_key.as_ref() {
        validate_parameter_key(parameter_key)?;

        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_config.parameter_key = parameter_key.clone();
        response = response.add_attribute("is_updated_parameter_key", "true");
    }

    if let Some(deposit_amount) = deposit_amount {
        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_config.deposit_amount = deposit_amount;
        response = response.add_attribute("is_updated_deposit_amount", "true");
    }

    if let Some(deposit_lock_period) = deposit_lock_period {
        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_config.deposit_lock_period = deposit_lock_period;
        response = response.add_attribute("is_updated_deposit_lock_period", "true");
    }

    if let Some(qualifier) = qualifier.as_ref() {
        campaign_config.qualifier = Some(deps.api.addr_validate(qualifier)?);
        response = response.add_attribute("is_updated_qualifier", "true");
    }

    if let Some(qualification_description) = qualification_description {
        validate_qualification_description(&qualification_description)?;
        campaign_config.qualification_description = Some(qualification_description);
        response = response.add_attribute("is_updated_qualification_description", "true");
    }

    if let Some(admin) = admin.as_ref() {
        CampaignConfig::save_admin_nominee(deps.storage, &deps.api.addr_validate(admin)?)?;
        response = response.add_attribute("is_updated_admin_nominee", "true");
    }

    campaign_config.save(deps.storage)?;

    Ok(response)
}

pub fn approve_admin_nominee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> ContractResult<Response> {
    // Validate
    let mut campaign_config = CampaignConfig::load(deps.storage)?;
    if !campaign_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("approve_admin_nominee");

    let address = deps.api.addr_validate(address.as_str())?;
    if let Some(admin_nominee) = CampaignConfig::may_load_admin_nominee(deps.storage)? {
        if admin_nominee != address {
            return Err(ContractError::Std(StdError::generic_err("It is not admin nominee")));
        }
    }

    campaign_config.admin = address;
    response = response.add_attribute("is_updated_admin", "true");

    campaign_config.save(deps.storage)?;

    Ok(response)
}

pub fn update_reward_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    participation_reward_amount: Option<Uint128>,
    participation_reward_lock_period: Option<u64>,
    referral_reward_amounts: Option<Vec<Uint128>>,
    referral_reward_lock_period: Option<u64>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    if !is_pending(deps.storage)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Only modifiable in pending status",
        )));
    }

    // Execute
    let mut response = make_response("update_reward_config");

    let mut reward_config = RewardConfig::load(deps.storage)?;

    if let Some(participation_reward_amount) = participation_reward_amount {
        reward_config.participation_reward_amount = participation_reward_amount;
        response = response.add_attribute("is_updated_participation_reward_amount", "true");
    }

    if let Some(participation_reward_lock_period) = participation_reward_lock_period {
        reward_config.participation_reward_lock_period = participation_reward_lock_period;
        response = response.add_attribute("is_updated_participation_reward_lock_period", "true");
    }

    if let Some(referral_reward_amounts) = referral_reward_amounts {
        reward_config.referral_reward_amounts = referral_reward_amounts;
        response = response.add_attribute("is_updated_referral_reward_amounts", "true");
    }

    if let Some(referral_reward_lock_period) = referral_reward_lock_period {
        reward_config.referral_reward_lock_period = referral_reward_lock_period;
        response = response.add_attribute("is_updated_referral_reward_lock_period", "true");
    }

    reward_config.save(deps.storage)?;

    Ok(response)
}

pub fn set_no_qualification(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // Validate
    let mut campaign_config = CampaignConfig::load(deps.storage)?;
    if !campaign_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("set_no_qualification");

    campaign_config.qualifier = None;
    response = response.add_attribute("is_updated_qualifier", "true");

    campaign_config.save(deps.storage)?;

    Ok(response)
}

pub fn update_activation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    is_active: bool,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign_state = CampaignState::load(deps.storage)?;

    if campaign_state.chain_id != env.block.chain_id {
        return Err(ContractError::Std(StdError::generic_err("Different chain_id. Required migrate contract.")));
    }

    // Execute
    let mut response = make_response("update_activation");

    campaign_state.active_flag = is_active;

    if is_active {
        campaign_state.last_active_height = Some(env.block.height);
    }

    campaign_state.save(deps.storage)?;

    response = response.add_attribute(
        "last_active_height",
        campaign_state.last_active_height
            .map_or(String::new(), |v| v.to_string()),
    );

    Ok(response)
}

pub fn add_reward_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    participation_reward_amount: Uint128,
    referral_reward_amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    let reward_config = RewardConfig::load(deps.storage)?;

    if !participation_reward_amount.is_zero() {
        if let cw20::Denom::Native(denom) = &reward_config.participation_reward_denom {
            validate_native_send(&info, denom, &participation_reward_amount)?;
        }
    }

    let campaign_config = CampaignConfig::load(deps.storage)?;
    let (key_denom, referral_reward_pool_ratio, add_pool_value) = validate_reward_pool_weight(
        &deps.querier,
        deps.api,
        &campaign_config,
        &reward_config,
        participation_reward_amount,
        referral_reward_amount,
    )?;

    // Execute
    let mut response = make_response("add_reward_pool");
    response = response.add_attribute("participation_reward_amount", participation_reward_amount.to_string());
    response = response.add_attribute("key_denom", Denom::from_cw20(key_denom).to_string());
    response = response.add_attribute("referral_reward_pool_ratio", referral_reward_pool_ratio.to_string());
    response = response.add_attribute("add_pool_value", add_pool_value);

    let global_campaign_config = load_global_campaign_config(&deps.querier, &campaign_config.campaign_manager)?;

    let add_pool_fee_amount = calc_add_pool_fee_amount(
        referral_reward_amount,
        referral_reward_pool_ratio,
        global_campaign_config.add_pool_fee_rate,
    )?;
    response = response.add_attribute("add_pool_fee_amount", add_pool_fee_amount.to_string());

    let real_referral_reward_amount = referral_reward_amount.checked_sub(add_pool_fee_amount)?;
    response = response.add_attribute("referral_reward_amount", real_referral_reward_amount.to_string());

    let mut campaign_state = CampaignState::load(deps.storage)?;

    campaign_state.deposit(
        &reward_config.participation_reward_denom,
        &participation_reward_amount,
    );
    campaign_state.deposit(
        &cw20::Denom::Cw20(reward_config.referral_reward_token.clone()),
        &real_referral_reward_amount,
    );

    campaign_state.save(deps.storage)?;

    // If participation reward denom is cw20, It will be send with this execute_msg.
    if !participation_reward_amount.is_zero() {
        if let cw20::Denom::Cw20(token) = &reward_config.participation_reward_denom {
            response = response.add_message(message_factories::wasm_execute(
                token,
                &Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: participation_reward_amount.clone(),
                },
            ));
        }
    }

    response = response.add_message(message_factories::wasm_execute(
        &reward_config.referral_reward_token,
        &Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: referral_reward_amount.clone(),
        },
    ));

    if !add_pool_fee_amount.is_zero() {
        response = response.add_message(message_factories::wasm_execute(
            &reward_config.referral_reward_token,
            &Cw20ExecuteMsg::Transfer {
                recipient: campaign_config.campaign_manager.to_string(),
                amount: add_pool_fee_amount,
            },
        ));
    }

    Ok(response)
}

const FRACTION: Uint128 = Uint128::new(1_000_000);

pub fn calc_add_pool_fee_amount(
    send_amount: Uint128,
    ratio: Decimal,
    fee_rate: Decimal,
) -> StdResult<Uint128> {
    send_amount.checked_mul(FRACTION * fee_rate)?
        .checked_div(FRACTION * ratio)
        .map_err(StdError::divide_by_zero)
}

fn validate_native_send(
    info: &MessageInfo,
    denom: &String,
    amount: &Uint128,
) -> StdResult<()> {
    match info.funds.len() {
        0 => return Err(StdError::generic_err("Empty funds")),
        1 => {
            if info.funds[0].denom != *denom {
                Err(StdError::generic_err("Invalid funds"))
            } else if info.funds[0].amount != *amount {
                Err(StdError::generic_err("Different funds and message"))
            } else {
                Ok(())
            }
        }
        _ => Err(StdError::generic_err("Too many funds")),
    }
}

fn validate_reward_pool_weight(
    querier: &QuerierWrapper,
    api: &dyn Api,
    campaign_config: &CampaignConfig,
    reward_config: &RewardConfig,
    participation_reward_amount: Uint128,
    referral_reward_amount: Uint128,
) -> StdResult<(cw20::Denom, Decimal, Uint128)> {
    let global_campaign_config = load_global_campaign_config(
        &querier,
        &campaign_config.campaign_manager,
    )?;
    let key_denom = global_campaign_config.key_denom.to_cw20(api);

    let participation_reward_value = swap_simulate(
        &querier,
        &global_campaign_config.terraswap_router,
        reward_config.participation_reward_denom.clone(),
        key_denom.clone(),
        participation_reward_amount,
    ).unwrap_or(Uint128::zero());

    let referral_reward_value = swap_simulate(
        &querier,
        &global_campaign_config.terraswap_router,
        cw20::Denom::Cw20(reward_config.referral_reward_token.clone()),
        key_denom.clone(),
        referral_reward_amount,
    )?;

    let referral_reward_pool_rate = Decimal::from_ratio(
        referral_reward_value,
        participation_reward_value + referral_reward_value,
    );

    if referral_reward_pool_rate < global_campaign_config.add_pool_min_referral_reward_rate {
        return Err(StdError::generic_err(format!(
            "Referral reward rate must be greater than {}",
            global_campaign_config.add_pool_min_referral_reward_rate.to_string(),
        )));
    }

    Ok((key_denom, referral_reward_pool_rate, participation_reward_value + referral_reward_value))
}

fn swap_simulate(
    querier: &QuerierWrapper,
    terraswap_router: &String,
    offer: cw20::Denom,
    ask: cw20::Denom,
    amount: Uint128,
) -> StdResult<Uint128> {
    if offer == ask {
        return Ok(amount);
    }

    let response: SimulateSwapOperationsResponse = querier.query_wasm_smart(
        terraswap_router,
        &QueryMsg::SimulateSwapOperations {
            offer_amount: amount,
            operations: vec![swap_operation(offer, ask)],
        },
    )?;

    Ok(response.amount)
}

fn swap_operation(offer: cw20::Denom, ask: cw20::Denom) -> SwapOperation {
    match offer {
        cw20::Denom::Native(offer_denom) => {
            match ask {
                cw20::Denom::Native(ask_denom) => SwapOperation::NativeSwap {
                    offer_denom,
                    ask_denom,
                },
                cw20::Denom::Cw20(ask_token) => SwapOperation::TerraSwap {
                    offer_asset_info: AssetInfo::NativeToken { denom: offer_denom },
                    ask_asset_info: AssetInfo::Token { contract_addr: ask_token.to_string() },
                },
            }
        }
        cw20::Denom::Cw20(offer_token) => {
            match ask {
                cw20::Denom::Native(ask_denom) => SwapOperation::TerraSwap {
                    offer_asset_info: AssetInfo::Token { contract_addr: offer_token.to_string() },
                    ask_asset_info: AssetInfo::NativeToken { denom: ask_denom },
                },
                cw20::Denom::Cw20(ask_token) => SwapOperation::TerraSwap {
                    offer_asset_info: AssetInfo::Token { contract_addr: offer_token.to_string() },
                    ask_asset_info: AssetInfo::Token { contract_addr: ask_token.to_string() },
                },
            }
        }
    }
}

pub fn remove_reward_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let campaign_config = CampaignConfig::load(deps.storage)?;
    if !campaign_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign_state = CampaignState::load(deps.storage)?;

    let denom_cw20 = denom.to_cw20(deps.api);
    let balance = campaign_state.balance(&denom_cw20);
    let campaign_balance = balance.total;
    let locked_balance = balance.locked;
    let free_balance = campaign_balance.checked_sub(locked_balance)?;
    let remove_amount = amount.unwrap_or(free_balance);

    if remove_amount.is_zero() || remove_amount > free_balance {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    // Execute
    let mut response = make_response("remove_reward_pool");
    response = response.add_attribute("prev_campaign_balance", campaign_balance);
    response = response.add_attribute("prev_locked_balance", locked_balance);

    let mut receive_amount = remove_amount;
    let mut remove_pool_fee_amount = Uint128::zero();
    if !campaign_state.is_pending() {
        let global_campaign_config = load_global_campaign_config(
            &deps.querier,
            &campaign_config.campaign_manager,
        )?;

        //destructuring assignments are unstable (https://github.com/rust-lang/rust/issues/71126)
        let (_remove_pool_fee_amount, _receive_amount) = calc_ratio_amount(
            remove_amount,
            global_campaign_config.remove_pool_fee_rate,
        );
        remove_pool_fee_amount = _remove_pool_fee_amount;
        receive_amount = _receive_amount;

        campaign_state.withdraw(&denom_cw20, &remove_pool_fee_amount)?;
        response = response.add_message(make_send_msg(
            &deps.querier,
            denom_cw20.clone(),
            remove_pool_fee_amount,
            &campaign_config.campaign_manager,
        )?);
    }

    campaign_state.withdraw(&denom_cw20, &receive_amount)?;
    response = response.add_message(make_send_msg(
        &deps.querier,
        denom_cw20,
        receive_amount,
        &info.sender,
    )?);

    campaign_state.validate_balance()?;
    campaign_state.save(deps.storage)?;

    response = response.add_attribute("receive_amount", receive_amount);
    response = response.add_attribute("remove_pool_fee_amount", remove_pool_fee_amount);

    Ok(response)
}

pub fn claim_participation_reward(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    // Validate
    let mut actor = Actor::may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::NotFound {})?;

    let reward_amount = actor.claim_participation_reward_amount(env.block.height);

    if reward_amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Not exist claimable participation reward")));
    }

    // Execute
    let mut response = make_response("claim_participation_reward");

    let reward_config = RewardConfig::load(deps.storage)?;
    let mut campaign_state = CampaignState::load(deps.storage)?;

    campaign_state.unlock_balance(&reward_config.participation_reward_denom, &reward_amount)?;
    campaign_state.withdraw(&reward_config.participation_reward_denom, &reward_amount)?;

    actor.save(deps.storage)?;
    campaign_state.save(deps.storage)?;

    response = response.add_message(make_send_msg(
        &deps.querier,
        reward_config.participation_reward_denom.clone(),
        reward_amount,
        &actor.address,
    )?);
    response = response.add_attribute(
        "amount",
        format!(
            "{}{}",
            reward_amount,
            Denom::from_cw20(reward_config.participation_reward_denom),
        ),
    );

    Ok(response)
}

pub fn claim_referral_reward(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    // Validate
    let mut actor = Actor::may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::NotFound {})?;

    let reward_amount = actor.claim_referral_reward_amount(env.block.height);

    if reward_amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Not exist claimable referral reward")));
    }

    // Execute
    let mut response = make_response("claim_referral_reward");

    let reward_config = RewardConfig::load(deps.storage)?;
    let mut campaign_state = CampaignState::load(deps.storage)?;

    campaign_state.unlock_balance(
        &cw20::Denom::Cw20(reward_config.referral_reward_token.clone()),
        &reward_amount,
    )?;
    campaign_state.withdraw(
        &cw20::Denom::Cw20(reward_config.referral_reward_token.clone()),
        &reward_amount,
    )?;

    actor.save(deps.storage)?;
    campaign_state.save(deps.storage)?;

    response = response.add_message(make_send_msg(
        &deps.querier,
        cw20::Denom::Cw20(reward_config.referral_reward_token),
        reward_amount,
        &actor.address,
    )?);
    response = response.add_attribute("amount", reward_amount);

    Ok(response)
}

pub const REPLY_QUALIFY_PARTICIPATION: u64 = 1;

pub fn participate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actor: String,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    // Validate
    let actor = deps.api.addr_validate(&actor)?;

    if actor != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let campaign_config = CampaignConfig::load(deps.storage)?;
    let campaign_state = CampaignState::load(deps.storage)?;
    if !campaign_state.is_active(&campaign_config, &deps.querier, &env.block)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Inactive campaign",
        )));
    }

    // Execute
    let mut response = make_response("participate");
    response = response.add_attribute("actor", actor.to_string());

    let referrer_address = referrer.and_then(|v| v.to_address(deps.api).ok());

    if let Some(qualifier) = campaign_config.qualifier {
        response = response.add_submessage(SubMsg {
            id: REPLY_QUALIFY_PARTICIPATION,
            msg: message_factories::wasm_execute(
                &qualifier,
                &QualifierExecuteMsg::Qualify(QualificationMsg {
                    campaign: env.contract.address.to_string(),
                    sender: info.sender.to_string(),
                    actor: actor.to_string(),
                    referrer: referrer_address.as_ref().map(|v| v.to_string()),
                }),
            ),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        });

        QualifyParticipationContext {
            actor: info.sender.clone(),
            referrer: referrer_address,
        }.save(deps.storage)?;
    } else {
        _participate(
            deps.storage,
            &deps.querier,
            &env,
            &mut response,
            actor,
            referrer_address,
            QualificationResult::success()
        )?;
    }

    Ok(response)
}

pub fn participate_qualify_result(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> ContractResult<Response> {
    let mut response = make_response("participate_qualify_result");

    if reply.result.is_err() {
        return Err(ContractError::Std(StdError::generic_err(reply.result.unwrap_err())));
    }

    let core_response: MsgExecuteContractResponse = Message::parse_from_bytes(
        reply.result.unwrap() .data.unwrap_or_default().as_slice(),
    ).map_err(|_| {
        StdError::parse_err("MsgExecuteContractResponse", "failed to parse data")
    })?;

    let result: QualificationResult = from_binary(&Binary(core_response.data))?;

    let context = QualifyParticipationContext::load(deps.storage)?;
    _participate(
        deps.storage,
        &deps.querier,
        &env,
        &mut response,
        context.actor,
        context.referrer,
        result,
    )?;

    QualifyParticipationContext::clear(deps.storage);

    Ok(response)
}

fn _participate(
    storage: &mut dyn Storage,
    querier: &QuerierWrapper,
    env: &Env,
    response: &mut Response,
    actor: Addr,
    referrer: Option<Addr>,
    qualify_result: QualificationResult,
) -> ContractResult<()> {
    if !qualify_result.can_participate {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Failed to qualify participation ({})", qualify_result.memo.unwrap_or_default()),
        )));
    }

    let mut my_participation = Actor::may_load(storage, &actor)?
        .unwrap_or_else(|| Actor::new(actor.clone(), referrer));

    my_participation.last_participated_at = env.block.time.clone();

    let campaign_config = CampaignConfig::load(storage)?;
    let mut campaign_state = CampaignState::load(storage)?;
    let reward_config = RewardConfig::load(storage)?;

    if campaign_config.require_deposit() {
        let mut deposit = Deposit::load_or_new(storage, &actor)?;

        let deposit_balance = deposit.balance(env.block.height)?;

        if deposit_balance < campaign_config.deposit_amount {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "Insufficient deposit balance (required: {}, current: {})",
                campaign_config.deposit_amount.to_string(),
                deposit_balance.to_string(),
            ))));
        }

        deposit.lock(campaign_config.deposit_amount, env.block.height, campaign_config.deposit_lock_period)?;
        deposit.save(storage)?;
    }

    let distributed_participation_reward_amount = distribute_participation_reward(
        &mut my_participation,
        &mut campaign_state,
        &reward_config,
        &qualify_result.participation_reward_rate,
        env,
    )?;

    let referral_reward_limit_option: ReferralRewardLimitOptionResponse = querier.query_wasm_smart(
        &campaign_config.campaign_manager,
        &valkyrie::campaign_manager::query_msgs::QueryMsg::ReferralRewardLimitOption {},
    )?;

    let (
        distributed_referral_reward_amount,
        referral_rewards,
        referral_reward_overflow_amount,
    ) = distribute_referral_reward(
        &mut my_participation,
        &mut campaign_state,
        &campaign_config,
        &reward_config,
        &qualify_result.referral_reward_rate,
        env,
        &referral_reward_limit_option,
        storage,
        querier,
    )?;

    if !referral_reward_overflow_amount.is_zero() {
        if let Some(recipient) = referral_reward_limit_option.overflow_amount_recipient {
            campaign_state.withdraw(
                &cw20::Denom::Cw20(reward_config.referral_reward_token.clone()),
                &referral_reward_overflow_amount,
            )?;

            response.messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_config.referral_reward_token.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient,
                    amount: referral_reward_overflow_amount,
                })?,
            })));
        }
    }

    //Check balance after distribute
    campaign_state.validate_balance().map_err(|_| StdError::generic_err("Insufficient balance"))?;
    let participation_reward_denom = Denom::from_cw20(reward_config.participation_reward_denom);

    response.attributes.push(attr(
        "distribute_result",
        to_binary(&DistributeResult {
            participation_reward_denom: participation_reward_denom.clone(),
            participation_reward_amount: distributed_participation_reward_amount,
            referral_rewards,
        })?.to_base64(),
    ));

    response.attributes.push(attr(
        "configured_participation_reward_amount",
        format!("{}{}",
                reward_config.participation_reward_amount.to_string(),
                participation_reward_denom.to_string(),
        ),
    ));
    response.attributes.push(attr(
        "distributed_participation_reward_amount",
        format!("{}{}",
                distributed_participation_reward_amount.to_string(),
                participation_reward_denom.to_string(),
        ),
    ));
    response.attributes.push(attr(
        "configured_referral_reward_amount",
        format!("{}{}",
                reward_config.referral_reward_amounts.iter().sum::<Uint128>().to_string(),
                reward_config.referral_reward_token.to_string(),
        ),
    ));
    response.attributes.push(attr(
        "distributed_referral_reward_amount",
        format!("{}{}",
                distributed_referral_reward_amount.to_string(),
                reward_config.referral_reward_token.to_string(),
        ),
    ));
    response.attributes.push(attr(
        "referral_reward_overflow_amount",
        format!("{}{}",
                referral_reward_overflow_amount.to_string(),
                reward_config.referral_reward_token.to_string(),
        ),
    ));

    if my_participation.participation_count == 1 {
        campaign_state.actor_count += 1;
    }

    campaign_state.participation_count += 1;
    campaign_state.last_active_height = Some(env.block.height);
    campaign_state.save(storage)?;
    my_participation.save(storage)?;

    response.attributes.push(attr(
        "cumulative_participation_reward_amount",
        campaign_state.cumulative_participation_reward_amount,
    ));
    response.attributes.push(attr(
        "cumulative_referral_reward_amount",
        campaign_state.cumulative_referral_reward_amount,
    ));
    response.attributes.push(attr("participation_count", campaign_state.actor_count.to_string()));
    response.attributes.push(attr("participate_count", campaign_state.participation_count.to_string()));

    Ok(())
}

fn distribute_participation_reward(
    actor: &mut Actor,
    campaign_state: &mut CampaignState,
    reward_config: &RewardConfig,
    reward_rate: &Decimal,
    env: &Env,
) -> StdResult<Uint128> {
    actor.participation_count += 1;

    let reward_amount = reward_config.participation_reward_amount * *reward_rate;

    actor.add_participation_reward(
        reward_amount,
        reward_config.participation_reward_lock_period + env.block.height,
    );
    actor.cumulative_participation_reward_amount += reward_amount;
    campaign_state.cumulative_participation_reward_amount += reward_amount;
    campaign_state.lock_balance(
        &reward_config.participation_reward_denom,
        &reward_amount,
    );

    Ok(reward_amount)
}

fn distribute_referral_reward(
    actor: &mut Actor,
    campaign_state: &mut CampaignState,
    campaign_config: &CampaignConfig,
    reward_config: &RewardConfig,
    reward_rate: &Decimal,
    env: &Env,
    referral_limit_option: &ReferralRewardLimitOptionResponse,
    storage: &mut dyn Storage,
    querier: &QuerierWrapper,
) -> StdResult<(Uint128, Vec<ReferralReward>, Uint128)> {
    let referral_reward_pool_balance = campaign_state
        .balance(&cw20::Denom::Cw20(reward_config.referral_reward_token.clone()))
        .available();

    if reward_config.referral_reward_amounts.iter().sum::<Uint128>() > referral_reward_pool_balance {
        return Ok((Uint128::zero(), vec![], Uint128::zero()));
    }

    let mut referrers = actor.load_referrers(
        storage,
        reward_config.referral_reward_amounts.len(),
    )?;

    if referrers.is_empty() {
        actor.referrer = None;

        return Ok((Uint128::zero(), vec![], Uint128::zero()));
    }

    let mut distributed_amount = Uint128::zero();
    let mut referral_rewards: Vec<ReferralReward> = vec![];
    let mut overflow_amount = Uint128::zero();

    let referrer_reward_pairs = referrers.iter_mut()
        .zip(&reward_config.referral_reward_amounts)
        .enumerate();

    let referral_reward_denom = cw20::Denom::Cw20(reward_config.referral_reward_token.clone());
    for (distance, (referrer_actor, reward_amount)) in referrer_reward_pairs {
        if actor.address == referrer_actor.address {
            return Err(StdError::generic_err("Actor must not contain on referrer chain"));
        }

        let reward_amount = *reward_amount * *reward_rate;
        let reward_limit = calc_referral_reward_limit(
            &referral_limit_option,
            &campaign_config,
            &reward_config,
            querier,
            &referrer_actor.address,
        )?.limit_amount;
        let mut actor_receive_amount = reward_amount;
        let mut actor_overflow_amount = Uint128::zero();
        let actor_reward_amount = referrer_actor.cumulative_referral_reward_amount + reward_amount;
        if reward_limit < actor_reward_amount {
            actor_overflow_amount = actor_reward_amount.checked_sub(reward_limit)?;
            actor_receive_amount = actor_receive_amount.checked_sub(actor_overflow_amount)
                .unwrap_or(Uint128::zero());
        }

        referrer_actor.referral_count += 1;
        referrer_actor.add_referral_reward(
            actor_receive_amount,
            reward_config.referral_reward_lock_period + env.block.height,
        );
        referrer_actor.cumulative_referral_reward_amount += actor_receive_amount;
        campaign_state.cumulative_referral_reward_amount += actor_receive_amount;
        campaign_state.lock_balance(&referral_reward_denom, &actor_receive_amount);
        distributed_amount += reward_amount;
        overflow_amount += actor_overflow_amount;

        referrer_actor.save(storage)?;

        referral_rewards.push(ReferralReward {
            address: referrer_actor.address.to_string(),
            distance: (distance + 1) as u64,
            amount: actor_receive_amount,
        });
    }

    Ok((distributed_amount, referral_rewards, overflow_amount))
}

pub fn deposit(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    sender: Addr,
    funds: Vec<(cw20::Denom, Uint128)>,
) -> ContractResult<Response> {
    if funds.len() < 1 {
        return Err(ContractError::Std(StdError::generic_err("Missing deposit denom")));
    } else if funds.len() > 1 {
        return Err(ContractError::Std(StdError::generic_err("Too many sent denom")));
    }

    let (send_denom, send_amount) = &funds[0];

    if send_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut response = Response::new();
    response = response.add_attribute("action", "deposit");

    let campaign_config = CampaignConfig::load(deps.storage)?;

    if let Some(deposit_denom) = campaign_config.deposit_denom {
        if *send_denom != deposit_denom {
            return Err(ContractError::Std(StdError::generic_err("Missing deposit denom")));
        }
    } else {
        return Err(ContractError::Std(StdError::generic_err("Missing deposit denom")));
    }

    let mut campaign_state = CampaignState::load(deps.storage)?;
    let mut deposit = Deposit::load_or_new(deps.storage, &sender)?;

    campaign_state.deposit_amount += send_amount;
    deposit.deposit_amount += send_amount;

    campaign_state.save(deps.storage)?;
    deposit.save(deps.storage)?;

    response = response.add_attribute("deposit", send_amount.to_string());
    response = response.add_attribute("balance", deposit.deposit_amount.to_string());

    Ok(response)
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let mut response = Response::new();
    response = response.add_attribute("action", "withdraw");

    let mut deposit = Deposit::load(deps.storage, &info.sender)?;

    response = response.add_attribute("deposit_amount", deposit.deposit_amount.to_string());
    response = response.add_attribute("locked_amount", deposit.locked_amount(env.block.height));

    deposit.clear(env.block.height);

    let balance = deposit.balance(env.block.height)?;

    if balance.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    if balance < amount {
        return Err(ContractError::Std(StdError::generic_err("Overdraw deposit")));
    }

    deposit.deposit_amount = deposit.deposit_amount.checked_sub(amount)?;

    deposit.save(deps.storage)?;

    let campaign_config = CampaignConfig::load(deps.storage)?;

    if let Some(denom) = campaign_config.deposit_denom {
        let mut campaign_state = CampaignState::load(deps.storage)?;

        campaign_state.deposit_amount = campaign_state.deposit_amount.checked_sub(amount)?;
        campaign_state.save(deps.storage)?;

        response = response.add_message(make_send_msg(
            &deps.querier,
            denom,
            amount,
            &info.sender,
        )?);
    } else {
        return Err(ContractError::Std(StdError::generic_err("No deposit")));
    }

    Ok(response)
}

fn validate_title(title: &str) -> StdResult<()> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(StdError::generic_err("Title too short"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(StdError::generic_err("Title too long"))
    } else {
        Ok(())
    }
}

fn validate_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Description too long"))
    } else {
        Ok(())
    }
}

fn validate_url(url: &str) -> StdResult<()> {
    if url.len() < MIN_URL_LENGTH {
        Err(StdError::generic_err("Url too short"))
    } else if url.len() > MAX_URL_LENGTH {
        Err(StdError::generic_err("Url too long"))
    } else {
        Ok(())
    }
}

fn validate_parameter_key(parameter_key: &str) -> StdResult<()> {
    if parameter_key.len() < MIN_PARAM_KEY_LENGTH {
        Err(StdError::generic_err("ParameterKey too short"))
    } else if parameter_key.len() > MAX_PARAM_KEY_LENGTH {
        Err(StdError::generic_err("ParameterKey too long"))
    } else {
        Ok(())
    }
}

fn validate_qualification_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Qualification description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Qualification description too long"))
    } else {
        Ok(())
    }
}

fn make_send_msg(
    querier: &QuerierWrapper,
    denom: Cw20Denom,
    amount_with_tax: Uint128,
    recipient: &Addr,
) -> StdResult<CosmosMsg> {
    match denom {
        Cw20Denom::Native(denom) => Ok(message_factories::native_send(
            querier,
            denom,
            recipient,
            amount_with_tax,
        )?),
        Cw20Denom::Cw20(contract_address) => Ok(message_factories::cw20_transfer(
            &contract_address,
            recipient,
            amount_with_tax,
        )),
    }
}

#[test]
fn test_validate_title() {
    assert_eq!(
        validate_title(
            &std::iter::repeat("X")
                .take(MIN_TITLE_LENGTH - 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Title too short"))
    );
    assert_eq!(
        validate_title(
            &std::iter::repeat("X")
                .take(MIN_TITLE_LENGTH + 1)
                .collect::<String>()
        ),
        Ok(())
    );
    assert_eq!(
        validate_title(
            &std::iter::repeat("X")
                .take(MAX_TITLE_LENGTH + 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Title too long"))
    );
}

#[test]
fn test_validate_description() {
    assert_eq!(
        validate_description(
            &std::iter::repeat("X")
                .take(MIN_DESC_LENGTH - 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Description too short"))
    );
    assert_eq!(
        validate_description(
            &std::iter::repeat("X")
                .take(MIN_DESC_LENGTH + 1)
                .collect::<String>()
        ),
        Ok(())
    );
    assert_eq!(
        validate_description(
            &std::iter::repeat("X")
                .take(MAX_DESC_LENGTH + 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Description too long"))
    );
}

#[test]
fn test_validate_url() {
    assert_eq!(
        validate_url(
            &std::iter::repeat("X")
                .take(MIN_URL_LENGTH - 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Url too short"))
    );
    assert_eq!(
        validate_url(
            &std::iter::repeat("X")
                .take(MIN_URL_LENGTH + 1)
                .collect::<String>()
        ),
        Ok(())
    );
    assert_eq!(
        validate_url(
            &std::iter::repeat("X")
                .take(MAX_URL_LENGTH + 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Url too long"))
    );
}

#[test]
fn test_validate_qualification_description() {
    assert_eq!(
        validate_qualification_description(
            &std::iter::repeat("X")
                .take(MIN_DESC_LENGTH - 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Qualification description too short"))
    );
    assert_eq!(
        validate_qualification_description(
            &std::iter::repeat("X")
                .take(MIN_DESC_LENGTH + 1)
                .collect::<String>()
        ),
        Ok(())
    );
    assert_eq!(
        validate_qualification_description(
            &std::iter::repeat("X")
                .take(MAX_DESC_LENGTH + 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Qualification description too long"))
    );
}
