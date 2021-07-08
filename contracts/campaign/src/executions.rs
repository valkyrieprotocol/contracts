use cosmwasm_std::{attr, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdError, StdResult, Uint128, WasmMsg, from_binary, Decimal};
use cw20::Denom as Cw20Denom;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::execute_msgs::{DistributeResult, Distribution, CampaignConfigMsg};
use valkyrie::common::{ContractResult, Denom, Execution, ExecutionMsg};
use valkyrie::cw20::query_balance;
use valkyrie::fund_manager::execute_msgs::{ExecuteMsg as FundExecuteMsg};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::calc_ratio_amount;

use crate::states::{BoosterState, CampaignInfo, CampaignState, ContractConfig, DistributionConfig, is_admin, is_pending, Participation, load_global_campaign_config, Booster, get_booster_id, DropBooster, ActivityBooster, PlusBooster, load_voting_power};
use valkyrie::campaign_manager::execute_msgs::{CampaignInstantiateMsg, ExecuteMsg};

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

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

    // Execute
    ContractConfig {
        admin: deps.api.addr_validate(&msg.admin)?,
        governance: deps.api.addr_validate(&msg.governance)?,
        campaign_manager: deps.api.addr_validate(&msg.campaign_manager)?,
        fund_manager: deps.api.addr_validate(&msg.fund_manager)?,
        proxies: msg.proxies.iter()
            .map(|v| deps.api.addr_validate(v).unwrap())
            .collect(),
    }.save(deps.storage)?;

    CampaignInfo {
        title: campaign_config.title,
        description: campaign_config.description,
        url: campaign_config.url,
        parameter_key: campaign_config.parameter_key,
        executions: msg.executions.iter()
            .map(|e| Execution::from(deps.api, e))
            .collect(),
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

    // Response
    Ok(Response::default())
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
    let mut contract_config = ContractConfig::load(deps.storage)?;

    if let Some(admin) = admin {
        contract_config.admin = deps.api.addr_validate(&admin)?;
    }

    if let Some(proxies) = proxies {
        contract_config.proxies = proxies.iter()
            .map(|v| deps.api.addr_validate(v).unwrap())
            .collect();
    }

    contract_config.save(deps.storage)?;

    // Response
    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "update_contract_config")],
        data: None,
    })
}

pub fn update_campaign_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    execution_msgs: Option<Vec<ExecutionMsg>>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut campaign_info = CampaignInfo::load(deps.storage)?;

    if let Some(title) = title {
        validate_title(&title)?;
        campaign_info.title = title;
    }

    if let Some(description) = description {
        validate_description(&description)?;
        campaign_info.description = description;
    }

    if let Some(url) = url {
        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_info.url = url;
    }

    if let Some(parameter_key) = parameter_key {
        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_info.parameter_key = parameter_key;
    }

    if let Some(execution_msgs) = execution_msgs {
        campaign_info.executions = execution_msgs.iter()
            .map(|e| Execution::from(deps.api, e))
            .collect();
    }

    campaign_info.save(deps.storage)?;

    // Response
    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "update_campaign_info")],
        data: None,
    })
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
    let mut distribution_config = DistributionConfig::load(deps.storage)?;

    distribution_config.denom = denom.to_cw20(deps.api);
    distribution_config.amounts = amounts;

    distribution_config.save(deps.storage)?;

    // Response
    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "update_distribution_config")],
        data: None,
    })
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

    // Execute
    let mut campaign_state = CampaignState::load(deps.storage)?;

    campaign_state.active_flag = is_active;

    if is_active {
        campaign_state.last_active_height = Some(env.block.height);
    }

    campaign_state.save(deps.storage)?;

    // Response
    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "update_activation")],
        data: None,
    })
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

    if withdraw_amount > free_balance {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    // Execute
    let global_campaign_config = load_global_campaign_config(
        &deps.querier,
        &contract_config.campaign_manager,
    )?;
    let (withdraw_fee_amount, receive_amount) = if campaign_state.is_pending() {
        (Uint128::zero(), withdraw_amount)
    } else {
        calc_ratio_amount(withdraw_amount, global_campaign_config.withdraw_fee_rate)
    };

    let mut messages: Vec<CosmosMsg> = vec![];
    if !withdraw_fee_amount.is_zero() {
        messages.push(make_send_msg(
            &deps.querier,
            denom_cw20.clone(),
            withdraw_fee_amount,
            &Addr::unchecked(global_campaign_config.withdraw_fee_recipient),
        )?);
    }

    if !receive_amount.is_zero() {
        messages.push(make_send_msg(
            &deps.querier,
            denom_cw20,
            receive_amount,
            &info.sender,
        )?);
    }

    // Response
    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![
            attr("action", "withdraw"),
            attr("receive_amount", format!("{}{}", receive_amount, denom)),
            attr("withdraw_fee_amount", format!("{}{}", withdraw_fee_amount, denom)),
        ],
        data: None,
    })
}

pub fn claim_reward(deps: DepsMut, _env: Env, info: MessageInfo) -> ContractResult<Response> {
    // Execute
    let mut messages: Vec<CosmosMsg> = vec![];

    let mut participation = Participation::load(deps.storage, &info.sender)?;

    if participation.has_reward() {
        let distribution_config = DistributionConfig::load(deps.storage)?;
        let mut campaign_state = CampaignState::load(deps.storage)?;

        let reward_amount = participation.receive_reward(&mut campaign_state)?;

        campaign_state.save(deps.storage)?;

        messages.push(make_send_msg(
            &deps.querier,
            distribution_config.denom.clone(),
            reward_amount,
            &participation.actor_address,
        )?);
    }

    let booster_state = BoosterState::load(deps.storage)?;
    if participation.has_booster_reward(booster_state.recent_booster_id) {
        let contract_config = ContractConfig::load(deps.storage)?;

        let reward_amount = participation.receive_booster_reward(
            deps.storage,
            booster_state.recent_booster_id,
        )?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_config.fund_manager.to_string(),
            send: vec![],
            msg: to_binary(&FundExecuteMsg::Transfer {
                recipient: participation.actor_address.to_string(),
                amount: reward_amount.clone(),
            }).unwrap(),
        }));
    }

    participation.save(deps.storage)?;

    // Response
    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![
            attr("action", "claim_reward"),
        ],
        data: None,
    })
}

pub fn participate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    let contract_config = ContractConfig::load(deps.storage)?;

    // Validate
    if !contract_config.can_participate_execution(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign_state = CampaignState::load(deps.storage)?;
    if !campaign_state.is_active(deps.storage, &deps.querier, env.block.height)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Inactive campaign",
        )));
    }

    if Participation::load(deps.storage, &info.sender).is_ok() {
        return Err(ContractError::Std(StdError::generic_err(
            "Already participated",
        )));
    }

    // Execute
    let distribution_config = DistributionConfig::load(deps.storage)?;
    let booster_state = BoosterState::load(deps.storage)?;
    let next_booster_id = booster_state.recent_booster_id + 1;
    let mut active_booster = Booster::may_load_active(deps.storage)?;

    let mut referrer = referrer.and_then(|v| v.to_address(deps.api).ok())
        .and_then(|a| Participation::may_load(deps.storage, &a).unwrap_or_default());

    let my_participation = Participation {
        actor_address: info.sender.clone(),
        referrer_address: referrer.clone().map(|r| r.actor_address),
        reward_amount: Uint128::zero(),
        participated_at: env.block.time.clone(),
        drop_booster_claimable: vec![(next_booster_id, true)],
        drop_booster_distance_counts: vec![],
        activity_booster_reward_amount: Uint128::zero(),
        plus_booster_reward_amount: Uint128::zero(),
    };

    let mut participations = vec![my_participation];
    let mut remain_distance = distribution_config.amounts.len() - 1;

    while referrer.is_some() && remain_distance > 0 {
        let referrer_participation = referrer.unwrap();

        referrer = referrer_participation.referrer_address.clone()
            .map(|a| Participation::load(deps.storage, &a).unwrap());

        participations.push(referrer_participation);
        remain_distance -= 1;
    }

    let distribution_denom = Denom::from_cw20(distribution_config.denom.clone());

    let mut distribution_amount = Uint128::zero();
    let total_activity_boost_amount = active_booster.as_ref()
        .map_or(Uint128::zero(), |b| b.activity_booster.reward_amount());
    let mut total_plus_boost_amount = Uint128::zero();
    let mut distributions_response: Vec<Distribution> = vec![];
    for (distance, (participation, reward_amount)) in participations
        .iter_mut()
        .zip(distribution_config.amounts.clone())
        .enumerate()
    {
        participation.reward_amount += reward_amount;
        participation.increase_distance_count(next_booster_id, distance as u64);
        campaign_state.increase_distance_count(distance as u64);
        campaign_state.plus_distribution(reward_amount);
        distribution_amount += reward_amount;

        let (activity_booster_amount, plus_booster_amount) = match active_booster.as_mut() {
            Some(booster) => {
                let activity_booster_amount = booster.activity_booster.boost(
                    participation,
                    distance as u64,
                    total_activity_boost_amount,
                );
                let plus_booster_amount = if distance == 0 {
                    booster.plus_booster.boost(
                        participation,
                        load_voting_power(
                            &deps.querier,
                            &contract_config.governance,
                            &participation.actor_address,
                        ),
                    )
                } else {
                    Uint128::zero()
                };

                (activity_booster_amount, plus_booster_amount)
            }
            None => {
                (Uint128::zero(), Uint128::zero())
            }
        };

        total_plus_boost_amount += plus_booster_amount;

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
        active_booster.activity_booster.left_amount().is_zero()
    } else {
        false
    };

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

    // Response
    let messages = if finish_booster {
        vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_config.campaign_manager.to_string(),
                send: vec![],
                msg: to_binary(&ExecuteMsg::FinishBoosting {
                    campaign: env.contract.address.to_string(),
                })?,
            }),
        ]
    } else {
        vec![]
    };

    let configured_reward = format!(
        "{}{}",
        distribution_config.amounts_sum().to_string(),
        Denom::from_cw20(distribution_config.denom.clone()).to_string()
    );

    let distributed_reward = format!(
        "{}{}",
        distribution_amount.to_string(),
        Denom::from_cw20(distribution_config.denom.clone()).to_string()
    );

    let result = DistributeResult {
        distributions: distributions_response,
    };

    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![
            attr("action", "participate"),
            attr("actor", info.sender),
            attr("configured_reward_amount", configured_reward),
            attr("distributed_reward_amount", distributed_reward),
            attr("activity_boost_amount", total_activity_boost_amount),
            attr("plus_boost_amount", total_plus_boost_amount),
        ],
        data: Some(to_binary(&result)?),
    })
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
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    if !contract_config.is_campaign_manager(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if Booster::is_boosting(deps.storage)? {
        return Err(ContractError::AlreadyExists {});
    }

    let distribution_config = DistributionConfig::load(deps.storage)?;
    let campaign_state: CampaignState = CampaignState::load(deps.storage)?;

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

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "enable_booster"),
            attr("booster_id", booster_id.to_string()),
            attr("drop_booster_amount", drop_booster_assigned_amount),
            attr("activity_booster_amount", activity_booster_assigned_amount),
            attr("plus_booster_amount", plus_booster_assigned_amount),
            attr("snapped_participation_count", campaign_state.participation_count),
        ],
        data: None,
    })
}

pub fn disable_booster(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult<Response> {
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    if !contract_config.is_campaign_manager(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut booster_state = Booster::load_active(deps.storage)?;
    booster_state.finish_with_save(deps.storage, env.block.time)?;

    let drop_booster_left_amount = booster_state.drop_booster.assigned_amount
        .checked_sub(booster_state.drop_booster.calculated_amount)?;

    let activity_booster_left_amount = booster_state.activity_booster.assigned_amount
        .checked_sub(booster_state.activity_booster.distributed_amount)?;

    let plus_booster_left_amount = booster_state.plus_booster.assigned_amount
        .checked_sub(booster_state.plus_booster.distributed_amount)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "disable_booster"),
            attr("drop_booster_left_amount", drop_booster_left_amount),
            attr("activity_booster_left_amount", activity_booster_left_amount),
            attr("plus_booster_left_amount", plus_booster_left_amount),
        ],
        data: None,
    })
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

fn validate_url(url: &str) -> StdResult<()> {
    //TODO: VALIDATE URL FORMAT
    if url.len() < MIN_LINK_LENGTH {
        Err(StdError::generic_err("Url too short"))
    } else if url.len() > MAX_LINK_LENGTH {
        Err(StdError::generic_err("Url too long"))
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
                .take(MIN_LINK_LENGTH - 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Url too short"))
    );
    assert_eq!(
        validate_url(
            &std::iter::repeat("X")
                .take(MIN_LINK_LENGTH + 1)
                .collect::<String>()
        ),
        Ok(())
    );
    assert_eq!(
        validate_url(
            &std::iter::repeat("X")
                .take(MAX_LINK_LENGTH + 1)
                .collect::<String>()
        ),
        Err(StdError::generic_err("Url too long"))
    );
}
