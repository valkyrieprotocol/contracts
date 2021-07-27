use cosmwasm_std::{to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdError, StdResult, Uint128, from_binary, Decimal, SubMsg};
use cw20::Denom as Cw20Denom;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::execute_msgs::{DistributeResult, Distribution, CampaignConfigMsg, MigrateMsg};
use valkyrie::common::{ContractResult, Denom, Execution, ExecutionMsg};
use valkyrie::cw20::query_balance;
use valkyrie::fund_manager::execute_msgs::{ExecuteMsg as FundExecuteMsg};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::{calc_ratio_amount, make_response};

use crate::states::{BoosterState, CampaignInfo, CampaignState, ContractConfig, DistributionConfig, is_admin, is_pending, Participation, load_global_campaign_config, Booster, get_booster_id, DropBooster, ActivityBooster, PlusBooster, load_voting_power};
use valkyrie::campaign_manager::execute_msgs::{CampaignInstantiateMsg, ExecuteMsg};

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

    // Execute
    let response = make_response("instantiate");

    ContractConfig {
        chain_id: env.block.chain_id,
        admin: deps.api.addr_validate(&msg.admin)?,
        governance: deps.api.addr_validate(&msg.governance)?,
        campaign_manager: deps.api.addr_validate(&msg.campaign_manager)?,
        fund_manager: deps.api.addr_validate(&msg.fund_manager)?,
        proxies: msg.proxies.iter()
            .map(|v| deps.api.addr_validate(v).unwrap())
            .collect(),
    }.save(deps.storage)?;

    let mut executions: Vec<Execution> = msg.executions.iter()
        .map(|e| Execution::from(deps.api, e))
        .collect();

    executions.sort_by_key(|e| e.order);

    CampaignInfo {
        title: campaign_config.title,
        description: campaign_config.description,
        url: campaign_config.url,
        parameter_key: campaign_config.parameter_key,
        executions,
        creator: deps.api.addr_validate(&msg.creator)?,
        created_at: env.block.time,
        created_height: env.block.height,
    }.save(deps.storage)?;

    CampaignState {
        participation_count: 0,
        distance_counts: vec![],
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        active_flag: false,
        last_active_height: None,
    }.save(deps.storage)?;

    DistributionConfig {
        denom: campaign_config.distribution_denom.to_cw20(deps.api),
        amounts: campaign_config.distribution_amounts,
    }.save(deps.storage)?;

    BoosterState {
        recent_booster_id: 0u64,
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
    response.add_attribute("chain_id", env.block.chain_id.clone());

    let mut contract_config = ContractConfig::load(deps.storage)?;

    contract_config.chain_id = env.block.chain_id;

    contract_config.save(deps.storage)?;

    Ok(response)
}

pub fn update_contract_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    proxies: Option<Vec<String>>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_contract_config");

    let mut contract_config = ContractConfig::load(deps.storage)?;

    if let Some(admin) = admin.as_ref() {
        contract_config.admin = deps.api.addr_validate(admin)?;
        response.add_attribute("is_updated_admin", "true");
    }

    if let Some(proxies) = proxies.as_ref() {
        contract_config.proxies = proxies.iter()
            .map(|v| deps.api.addr_validate(v).unwrap())
            .collect();
        response.add_attribute("is_updated_proxies", "true");
    }

    contract_config.save(deps.storage)?;

    Ok(response)
}

pub fn update_campaign_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    mut executions: Option<Vec<ExecutionMsg>>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_campaign_info");

    let mut campaign_info = CampaignInfo::load(deps.storage)?;

    if let Some(title) = title.as_ref() {
        validate_title(title)?;
        campaign_info.title = title.clone();
        response.add_attribute("is_updated_title", "true");
    }

    if let Some(description) = description.as_ref() {
        validate_description(description)?;
        campaign_info.description = description.clone();
        response.add_attribute("is_updated_description", "true");
    }

    if let Some(url) = url.as_ref() {
        validate_url(url)?;

        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_info.url = url.clone();
        response.add_attribute("is_updated_url", "true");
    }

    if let Some(parameter_key) = parameter_key.as_ref() {
        validate_parameter_key(parameter_key)?;

        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_info.parameter_key = parameter_key.clone();
        response.add_attribute("is_updated_parameter_key", "true");
    }

    if let Some(executions) = executions.as_mut() {
        executions.sort_by_key(|e| e.order);
        campaign_info.executions = executions.iter()
            .map(|e| Execution::from(deps.api, e))
            .collect();
        response.add_attribute("is_updated_executions", "true");
    }

    campaign_info.save(deps.storage)?;

    Ok(response)
}

pub fn update_distribution_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: Denom,
    amounts: Vec<Uint128>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if !is_pending(deps.storage)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Only modifiable in pending status",
        )));
    }

    // Execute
    let response = make_response("update_distribution_config");

    let mut distribution_config = DistributionConfig::load(deps.storage)?;

    distribution_config.denom = denom.to_cw20(deps.api);
    distribution_config.amounts = amounts;

    distribution_config.save(deps.storage)?;

    Ok(response)
}

pub fn update_activation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    is_active: bool,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let contract_config = ContractConfig::load(deps.storage)?;

    if contract_config.chain_id != env.block.chain_id {
        return Err(ContractError::Std(StdError::generic_err("Different chain_id. Required migrate contract.")));
    }

    // Execute
    let mut response = make_response("update_activation");

    let mut campaign_state = CampaignState::load(deps.storage)?;

    campaign_state.active_flag = is_active;

    if is_active {
        campaign_state.last_active_height = Some(env.block.height);
    }

    campaign_state.save(deps.storage)?;

        response.add_attribute(
            "last_active_height",
            campaign_state.last_active_height
                .map_or(String::new(), |v| v.to_string()),
        );

    Ok(response)
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let contract_config = ContractConfig::load(deps.storage)?;
    if !contract_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let distribution_config = DistributionConfig::load(deps.storage)?;
    let campaign_state = CampaignState::load(deps.storage)?;

    let campaign_balance = denom.load_balance(&deps.querier, deps.api, env.contract.address)?;
    let denom_cw20 = denom.to_cw20(deps.api);
    let locked_balance = if distribution_config.denom == denom_cw20 {
        campaign_state.locked_balance
    } else {
        Uint128::zero()
    };
    let free_balance = campaign_balance.checked_sub(locked_balance)?;
    let withdraw_amount = amount.unwrap_or(free_balance);

    if withdraw_amount.is_zero() || withdraw_amount > free_balance {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    // Execute
    let mut response = make_response("withdraw");
    response.add_attribute("pre_campaign_balance", campaign_balance);
    response.add_attribute("pre_locked_balance", locked_balance);

    let mut receive_amount = withdraw_amount;
    let mut withdraw_fee_amount = Uint128::zero();
    if !campaign_state.is_pending() {
        let global_campaign_config = load_global_campaign_config(
            &deps.querier,
            &contract_config.campaign_manager,
        )?;

        //destructuring assignments are unstable (https://github.com/rust-lang/rust/issues/71126)
        let (_withdraw_fee_amount, _receive_amount) = calc_ratio_amount(
            withdraw_amount,
            global_campaign_config.withdraw_fee_rate,
        );
        withdraw_fee_amount = _withdraw_fee_amount;
        receive_amount = _receive_amount;

        response.add_message(make_send_msg(
            &deps.querier,
            denom_cw20.clone(),
            withdraw_fee_amount,
            &Addr::unchecked(global_campaign_config.withdraw_fee_recipient),
        )?);
    }

    response.add_message(make_send_msg(
        &deps.querier,
        denom_cw20,
        receive_amount,
        &info.sender,
    )?);

    response.add_attribute("receive_amount", receive_amount);
    response.add_attribute("withdraw_fee_amount", withdraw_fee_amount);

    Ok(response)
}

pub fn claim_participation_reward(deps: DepsMut, _env: Env, info: MessageInfo) -> ContractResult<Response> {
    // Validate
    let participation = Participation::may_load(deps.storage, &info.sender)?;

    if participation.is_none() {
        return Err(ContractError::NotFound {});
    }

    let mut participation = participation.unwrap();

    if !participation.has_reward() {
        return Err(ContractError::Std(StdError::generic_err("Not exist participation reward")));
    }

    // Execute
    let mut response = make_response("claim_participation_reward");

    let distribution_config = DistributionConfig::load(deps.storage)?;
    let mut campaign_state = CampaignState::load(deps.storage)?;

    let reward_amount = participation.receive_reward(&mut campaign_state)?;

    campaign_state.save(deps.storage)?;
    participation.save(deps.storage)?;

    response.add_message(make_send_msg(
        &deps.querier,
        distribution_config.denom.clone(),
        reward_amount,
        &participation.actor_address,
    )?);
    response.add_attribute("amount", format!("{}{}", reward_amount, Denom::from_cw20(distribution_config.denom)));

    Ok(response)
}

pub fn claim_booster_reward(deps: DepsMut, _env: Env, info: MessageInfo) -> ContractResult<Response> {
    // Validate
    let booster_state = BoosterState::load(deps.storage)?;
    if booster_state.recent_booster_id == 0 {
        return Err(ContractError::Std(StdError::generic_err("Not boosted campaign")));
    }

    let participation = Participation::may_load(deps.storage, &info.sender)?;

    if participation.is_none() {
        return Err(ContractError::NotFound {});
    }

    let mut participation = participation.unwrap();

    if !participation.has_booster_reward(booster_state.recent_booster_id) {
        return Err(ContractError::Std(StdError::generic_err("Not exist booster reward")));
    }

    // Execute
    let mut response = make_response("claim_booster_reward");

    let contract_config = ContractConfig::load(deps.storage)?;

    let reward_amount = participation.receive_booster_reward(
        deps.storage,
        booster_state.recent_booster_id,
    )?;

    participation.save(deps.storage)?;

    response.add_message(message_factories::wasm_execute(
        &contract_config.fund_manager,
        &FundExecuteMsg::Transfer {
            recipient: participation.actor_address.to_string(),
            amount: reward_amount,
        }
    ));
    response.add_attribute("amount", reward_amount);

    Ok(response)
}

pub fn participate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actor: String,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    // Validate
    let contract_config = ContractConfig::load(deps.storage)?;
    let actor = deps.api.addr_validate(&actor)?;

    if !contract_config.can_participate_execution(&info.sender, &actor) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign_state = CampaignState::load(deps.storage)?;
    if !campaign_state.is_active(deps.storage, &deps.querier, &env.block)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Inactive campaign",
        )));
    }

    if Participation::load(deps.storage, &actor).is_ok() {
        return Err(ContractError::Std(StdError::generic_err(
            "Already participated",
        )));
    }

    // Execute
    let mut response = make_response("participate");

    let distribution_config = DistributionConfig::load(deps.storage)?;
    let booster_state = BoosterState::load(deps.storage)?;
    let next_booster_id = booster_state.recent_booster_id + 1;
    let mut active_booster = Booster::may_load_active(deps.storage)?;

    let mut my_participation = Participation {
        actor_address: actor.clone(),
        referrer_address: referrer.and_then(|v| v.to_address(deps.api).ok()),
        reward_amount: Uint128::zero(),
        participated_at: env.block.time.clone(),
        drop_booster_claimable: vec![(next_booster_id, true)],
        drop_booster_distance_counts: vec![],
        activity_booster_reward_amount: Uint128::zero(),
        plus_booster_reward_amount: Uint128::zero(),
    };

    let mut participations = my_participation.load_referrers(
        deps.storage,
        distribution_config.amounts.len() - 1,
    )?;
    if participations.is_empty() {
        my_participation.referrer_address = None;
    }
    participations.insert(0, my_participation);

    let mut total_reward_amount = Uint128::zero();
    let mut total_activity_booster_amount = Uint128::zero();
    let mut total_plus_booster_amount = Uint128::zero();
    let distribution_denom = Denom::from_cw20(distribution_config.denom.clone());

    let mut distributions_response: Vec<Distribution> = vec![];
    let participation_rewards = participations.iter_mut()
        .zip(distribution_config.amounts.clone())
        .enumerate();

    for (distance, (participation, reward_amount)) in participation_rewards {
        //Distribute reward
        participation.reward_amount += reward_amount;
        participation.increase_distance_count(next_booster_id, distance as u64);
        campaign_state.increase_distance_count(distance as u64);
        campaign_state.plus_distribution(reward_amount);
        total_reward_amount += reward_amount;

        //Distribute booster
        let mut activity_booster_amount = Uint128::zero();
        let mut plus_booster_amount = Uint128::zero();

        if let Some(booster) = active_booster.as_mut() {
            activity_booster_amount = booster.activity_booster.boost(
                participation,
                distance as u64,
            );

            total_activity_booster_amount += activity_booster_amount;

            if distance == 0 {
                plus_booster_amount = booster.plus_booster.boost(
                    participation,
                    load_voting_power(
                        &deps.querier,
                        &contract_config.governance,
                        &participation.actor_address,
                    ),
                );

                total_plus_booster_amount += plus_booster_amount;
            }
        }

        //Save
        participation.save(deps.storage)?;

        distributions_response.push(Distribution {
            address: participation.actor_address.to_string(),
            distance: distance as u64,
            reward_denom: distribution_denom.clone(),
            reward_amount,
            activity_boost_amount: activity_booster_amount,
            plus_boost_amount: plus_booster_amount,
        });
    }

    campaign_state.participation_count += 1;
    campaign_state.last_active_height = Some(env.block.height);
    campaign_state.save(deps.storage)?;

    let finish_booster = if let Some(active_booster) = active_booster {
        active_booster.save(deps.storage)?;
        !active_booster.can_boost()
    } else {
        false
    };

    //Check balance after distribute
    let campaign_balance = query_balance(
        &deps.querier,
        deps.api,
        distribution_config.denom.clone(),
        env.contract.address.clone(),
    )?;

    if campaign_state.locked_balance > campaign_balance {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    let campaign_info = CampaignInfo::load(deps.storage)?;
    for execution in campaign_info.executions.iter() {
        response.add_message(message_factories::wasm_execute_bin(
            &execution.contract,
            execution.msg.clone(),
        ));
    }

    if finish_booster {
        response.messages.insert(0, SubMsg::new(message_factories::wasm_execute(
            &contract_config.campaign_manager,
            &ExecuteMsg::FinishBoosting {
                campaign: env.contract.address.to_string(),
            },
        )));
    }

    let configured_reward = format!(
        "{}{}",
        distribution_config.amounts_sum().to_string(),
        Denom::from_cw20(distribution_config.denom.clone()).to_string()
    );

    let distributed_reward = format!(
        "{}{}",
        total_reward_amount.to_string(),
        Denom::from_cw20(distribution_config.denom.clone()).to_string()
    );

    response.add_attribute("actor", actor.to_string());
    response.add_attribute("booster_id", booster_state.recent_booster_id.to_string());
    response.add_attribute("configured_reward_amount", configured_reward);
    response.add_attribute("distributed_reward_amount", distributed_reward);
    response.add_attribute("activity_boost_amount", total_activity_booster_amount);
    response.add_attribute("plus_boost_amount", total_plus_booster_amount);
    response.add_attribute("distribution_length", distributions_response.len().to_string());
    response.add_attribute("participation_count", campaign_state.participation_count.to_string());
    response.add_attribute("cumulative_distribution_amount", campaign_state.cumulative_distribution_amount);
    response.add_attribute("finish_booster", finish_booster.to_string());

    response.set_data(to_binary(&DistributeResult {
        distributions: distributions_response,
    })?);

    Ok(response)
}

pub fn enable_booster(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    drop_booster_assigned_amount: Uint128,
    activity_booster_assigned_amount: Uint128,
    plus_booster_assigned_amount: Uint128,
    activity_booster_multiplier: Decimal,
) -> ContractResult<Response> {
    // Validate
    if drop_booster_assigned_amount.is_zero() || activity_booster_assigned_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    if !contract_config.is_campaign_manager(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if Booster::is_boosting(deps.storage)? {
        return Err(ContractError::AlreadyExists {});
    }

    let distribution_config = DistributionConfig::load(deps.storage)?;
    let campaign_state: CampaignState = CampaignState::load(deps.storage)?;

    if campaign_state.participation_count == 0 {
        return Err(ContractError::Std(StdError::generic_err("participation_count must be greater than 0")));
    }

    // Execute
    let mut response = make_response("enable_booster");

    let booster_id = get_booster_id(deps.storage)?;

    let drop_booster = DropBooster::new(
        drop_booster_assigned_amount,
        distribution_config.amounts.clone(),
        campaign_state.participation_count,
        campaign_state.distance_counts,
    );

    let activity_booster = ActivityBooster::new(
        activity_booster_assigned_amount,
        distribution_config.amounts.clone(),
        drop_booster.reward_amount.clone(),
        activity_booster_multiplier,
    );

    let plus_booster = PlusBooster::new(
        plus_booster_assigned_amount,
    );

    Booster {
        id: booster_id,
        drop_booster,
        activity_booster,
        plus_booster,
        boosted_at: env.block.time,
        finished_at: None,
    }.save(deps.storage)?;

    response.add_attribute("booster_id", booster_id.to_string());
    response.add_attribute("drop_booster_amount", drop_booster_assigned_amount);
    response.add_attribute("activity_booster_amount", activity_booster_assigned_amount);
    response.add_attribute("plus_booster_amount", plus_booster_assigned_amount);
    response.add_attribute("activity_booster_multiplier", activity_booster_multiplier.to_string());
    response.add_attribute("snapped_participation_count", campaign_state.participation_count.to_string());

    Ok(response)
}

pub fn disable_booster(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    // Validate
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    if !contract_config.is_campaign_manager(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("disable_booster");

    let mut booster_state = Booster::load_active(deps.storage)?;
    booster_state.finish_with_save(deps.storage, env.block.time)?;

    let drop_booster_left_amount = booster_state.drop_booster.assigned_amount
        .checked_sub(booster_state.drop_booster.calculated_amount)?;

    let activity_booster_left_amount = booster_state.activity_booster.assigned_amount
        .checked_sub(booster_state.activity_booster.distributed_amount)?;

    let plus_booster_left_amount = booster_state.plus_booster.assigned_amount
        .checked_sub(booster_state.plus_booster.distributed_amount)?;

    response.add_attribute("drop_booster_left_amount", drop_booster_left_amount);
    response.add_attribute("activity_booster_left_amount", activity_booster_left_amount);
    response.add_attribute("plus_booster_left_amount", plus_booster_left_amount);

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
