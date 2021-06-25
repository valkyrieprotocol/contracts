use cosmwasm_std::{
    attr, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, Uint64, WasmMsg
};
use cw20::Denom;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::campaign::execute_msgs::{DistributeResult, Distribution, InstantiateMsg};
use valkyrie::common::ContractResult;
use valkyrie::cw20::query_balance;
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::{calc_ratio_amount, find, map_u128};
use valkyrie::distributor::execute_msgs::ExecuteMsg as DistributorExecuteMsg;

use crate::states::{
    is_admin, is_pending, load_valkyrie_config, BoosterState, CampaignInfo, CampaignState,
    ContractConfig, DistributionConfig, Participation,
};

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // Validate
    validate_title(&msg.title)?;
    validate_url(&msg.url)?;
    validate_description(&msg.description)?;

    // Execute
    ContractConfig {
        admin: info.sender.clone(),
        governance: deps.api.addr_validate(&msg.governance)?,
        distributor: deps.api.addr_validate(&msg.distributor)?,
        token_contract: deps.api.addr_validate(&msg.token_contract)?,
    }
    .save(deps.storage)?;

    CampaignInfo {
        title: msg.title,
        description: msg.description,
        url: msg.url,
        parameter_key: msg.parameter_key,
        creator: info.sender.clone(),
        created_at: env.block.time,
        created_block: env.block.height,
    }
    .save(deps.storage)?;

    CampaignState {
        participation_count: 0,
        cumulative_distribution_amount: vec![],
        locked_balance: vec![],
        active_flag: false,
        last_active_block: None,
    }
    .save(deps.storage)?;

    DistributionConfig {
        denom: msg.distribution_denom.to_cw20(deps.api),
        amounts: msg.distribution_amounts,
    }
    .save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn update_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    url: Option<String>,
    title: Option<String>,
    description: Option<String>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut campaign_info = CampaignInfo::load(deps.storage)?;

    if let Some(url) = url {
        if !is_pending(deps.storage)? {
            return Err(ContractError::Std(StdError::generic_err(
                "Only modifiable in pending status",
            )));
        }

        campaign_info.url = url;
    }

    if let Some(title) = title {
        validate_title(&title)?;
        campaign_info.title = title;
    }

    if let Some(description) = description {
        validate_description(&description)?;
        campaign_info.description = description;
    }

    campaign_info.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn update_distribution_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: valkyrie::campaign::enumerations::Denom,
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
    Ok(Response::default())
}

pub fn update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut contract_config = ContractConfig::load(deps.storage)?;

    contract_config.admin = deps.api.addr_validate(&address)?;

    contract_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
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
        campaign_state.last_active_block = Some(env.block.height);
    }

    campaign_state.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn withdraw_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: valkyrie::campaign::enumerations::Denom,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let contract_config = ContractConfig::load(deps.storage)?;
    if !contract_config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let campaign_state = CampaignState::load(deps.storage)?;

    let campaign_balance = denom.load_balance(&deps.querier, deps.api, env.contract.address)?;
    let free_balance = campaign_balance
        - campaign_state
            .locked_balance(denom.to_cw20(deps.api))
            .u128();
    let withdraw_amount = amount.map_or_else(|| free_balance, |v| v.u128());

    if withdraw_amount > free_balance {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    // Execute
    let valkyrie_config = load_valkyrie_config(&deps.querier, &contract_config.governance)?;
    let (burn_amount, receive_amount) = if campaign_state.is_pending() {
        (0u128, withdraw_amount)
    } else {
        calc_ratio_amount(withdraw_amount, valkyrie_config.reward_withdraw_burn_rate)
    };

    let denom_cw20 = denom.to_cw20(deps.api);
    let burn_msg = make_send_msg(
        denom_cw20.clone(),
        burn_amount,
        &Addr::unchecked(valkyrie_config.burn_contract), //valkyrie_config 에 저장할 때 유효성 검사하므로 여기서는 하지 않음.
    );
    let send_msg = make_send_msg(denom_cw20, receive_amount, &info.sender);

    // Response
    Ok(Response {
        submessages: vec![],
        messages: vec![burn_msg, send_msg],
        attributes: vec![],
        data: None,
    })
}

pub fn claim_reward(deps: DepsMut, _env: Env, info: MessageInfo) -> ContractResult<Response> {
    let contract_config = ContractConfig::load(deps.storage)?;

    // Execute
    let mut messages: Vec<CosmosMsg> = vec![];

    let mut campaign_state = CampaignState::load(deps.storage)?;
    let mut participation = Participation::load(deps.storage, &info.sender)?;

    // normal rewards
    for (denom, amount) in participation.rewards {
        campaign_state.unlock_balance(denom.clone(), amount)?;
        messages.push(make_send_msg(denom, amount.u128(), &info.sender));
    }

    participation.rewards = vec![];

    // check drop booster
    if participation.drop_booster_claimable {
        let drop_booster = BoosterState::compute_and_spend_drop_booster(deps.storage)?;
        participation.booster_rewards += drop_booster;
        participation.drop_booster_claimable = false;
    }

    // claim booster rewards
    let booster_claim_amount = participation.booster_rewards;
    if !booster_claim_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_config.distributor.to_string(),
            send: vec![],
            msg: to_binary(&DistributorExecuteMsg::Spend {
                recipient: info.sender.to_string(),
                amount: booster_claim_amount,
            })?,
        }))
    }

    participation.booster_rewards = Uint128::zero();
    participation.save(deps.storage)?;

    // Response
    Ok(Response {
        submessages: vec![],
        messages,
        attributes: vec![],
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
    let mut campaign_state = CampaignState::load(deps.storage)?;
    if !campaign_state.is_active(deps.storage, &deps.querier, env.block.height)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Deactivated campaign",
        )));
    }

    if Participation::load(deps.storage, &info.sender).is_ok() {
        return Err(ContractError::Std(StdError::generic_err(
            "Already participated",
        )));
    }

    // Execute
    let distribution_config = DistributionConfig::load(deps.storage)?;
    let (activity_booster, plus_booster, drop_booster_claimable) =
        BoosterState::compute_and_spend_participate_booster(
            deps.storage,
            &deps.querier,
            &contract_config.governance,
            &info.sender,
        )?;

    let mut referrer = if referrer.is_some() {
        referrer.unwrap().to_address(deps.api).ok() // Ignore wrong referrer
    } else {
        None
    };

    let my_participation = Participation {
        actor_address: info.sender.clone(),
        referrer_address: referrer.clone(),
        rewards: vec![],
        booster_rewards: plus_booster,
        drop_booster_claimable,
    };

    let mut participations = vec![my_participation];
    let mut remain_distance = distribution_config.amounts.len() - 1;

    while referrer.is_some() && remain_distance > 0 {
        let participation = Participation::load(deps.storage, &referrer.unwrap())?;
        referrer = participation.referrer_address.clone();
        participations.push(participation);
        remain_distance -= 1;
    }

    let reward_amount_sum: Uint128 = distribution_config.amounts_sum();
    let mut distributions: Vec<(Addr, Vec<(Denom, u128)>)> = vec![];
    for (participation, reward_amount) in participations
        .iter_mut()
        .zip(distribution_config.amounts.clone())
    {
        // activity booster is distributed 
        // in the same ratio with normal rewards scheme
        let booster_rewards = Uint128::from(
            activity_booster.u128() * reward_amount.u128() / reward_amount_sum.u128(),
        );

        participation.plus_reward(distribution_config.denom.clone(), reward_amount);
        participation.booster_rewards += booster_rewards;
        participation.save(deps.storage)?;

        campaign_state.plus_distribution(distribution_config.denom.clone(), reward_amount);

        distributions.push((
            participation.actor_address.clone(),
            vec![
                vec![(distribution_config.denom.clone(), reward_amount.u128())],
                if booster_rewards.is_zero() {
                    vec![]
                } else {
                    vec![(
                        cw20::Denom::Cw20(contract_config.token_contract.clone()),
                        booster_rewards.u128(),
                    )]
                },
            ]
            .concat(),
        ))
    }

    campaign_state.participation_count += 1;
    campaign_state.last_active_block = Some(env.block.height);
    campaign_state.save(deps.storage)?;

    let campaign_balance = query_balance(
        &deps.querier,
        deps.api,
        distribution_config.denom.clone(),
        env.contract.address,
    )?;

    if campaign_state
        .locked_balance(distribution_config.denom.clone())
        .u128()
        > campaign_balance
    {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    // Response
    let mut distribution_amount = Uint128::zero();
    let mut distributions_response: Vec<Distribution> = vec![];

    for (index, (address, rewards)) in distributions.iter().enumerate() {
        let amount = find(rewards, |(denom, _)| distribution_config.denom.eq(denom))
            .map_or(Uint128::zero(), |(_, amount)| Uint128::from(*amount));

        distribution_amount += amount;
        distributions_response.push(Distribution {
            address: address.to_string(),
            distance: Uint64::from(index as u64),
            amount,
        });
    }

    let result = DistributeResult {
        actor_address: info.sender.to_string(),
        reward_denom: valkyrie::campaign::enumerations::Denom::from_cw20(
            distribution_config.denom.clone(),
        ),
        configured_reward_amount: Uint128::new(map_u128(distribution_config.amounts).iter().sum()),
        distributed_reward_amount: distribution_amount,
        distributions: distributions_response,
    };

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![],
        data: Some(to_binary(&result)?),
    })
}

pub fn register_booster(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    drop_booster_amount: Uint128,
    activity_booster_amount: Uint128,
    plus_booster_amount: Uint128,
) -> ContractResult<Response> {
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    if contract_config.is_distributor(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if BoosterState::load(deps.storage).is_ok() {
        return Err(ContractError::AlreadyExists {});
    }

    let campaign_state: CampaignState = CampaignState::load(deps.storage)?;
    let booster_state = BoosterState {
        drop_booster_amount,
        drop_booster_left_amount: drop_booster_amount,
        drop_booster_participations: campaign_state.participation_count,
        activity_booster_amount,
        activity_booster_left_amount: activity_booster_amount,
        plus_booster_amount,
        plus_booster_left_amount: plus_booster_amount,
    };

    booster_state.save(deps.storage)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "register_booster"),
            attr("drop_booster_amount", drop_booster_amount),
            attr(
                "drop_booster_participations",
                campaign_state.participation_count,
            ),
            attr("activity_booster_amount", activity_booster_amount),
            attr("plus_booster_amount", plus_booster_amount),
        ],
        data: None,
    })
}

pub fn deregister_booster(deps: DepsMut, _env: Env, info: MessageInfo) -> ContractResult<Response> {
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    if contract_config.is_distributor(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let booster_state = BoosterState::load(deps.storage)?;
    BoosterState::remove(deps.storage);

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "deregister_booster"),
            attr(
                "drop_booster_left_amount",
                booster_state.drop_booster_left_amount,
            ),
            attr(
                "activity_booster_left_amount",
                booster_state.activity_booster_left_amount,
            ),
            attr(
                "plus_booster_left_amount",
                booster_state.plus_booster_left_amount,
            ),
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

fn make_send_msg(denom: Denom, amount_with_tax: u128, recipient: &Addr) -> CosmosMsg {
    match denom {
        Denom::Native(denom) => {
            message_factories::native_send(denom, recipient, Uint128::from(amount_with_tax))
        }
        Denom::Cw20(contract_address) => message_factories::cw20_transfer(
            &contract_address,
            recipient,
            Uint128::from(amount_with_tax),
        ),
    }
}
