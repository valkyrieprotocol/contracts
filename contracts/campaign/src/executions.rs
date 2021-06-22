use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128, CosmosMsg, Addr, StdError, StdResult};
use valkyrie::common::ContractResult;
use valkyrie::campaign::execute_msgs::InstantiateMsg;
use valkyrie::message_factories;
use crate::states::{CampaignInfo, DistributionConfig, CampaignState, is_governance, ContractConfig, is_admin, Participation};
use cw20::Denom;
use valkyrie::errors::ContractError;
use valkyrie::utils::{map_u128, calc_ratio_amount};
use valkyrie::cw20::query_balance;
use valkyrie::campaign::enumerations::Referrer;
use valkyrie::governance::query_msgs::ValkyrieConfigResponse;


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
    }.save(deps.storage)?;

    CampaignInfo {
        title: msg.title,
        description: msg.description,
        url: msg.url,
        parameter_key: msg.parameter_key,
        creator: info.sender.clone(),
        created_at: env.block.time,
        created_block: env.block.height,
    }.save(deps.storage)?;

    CampaignState {
        participation_count: 0,
        cumulative_distribution_amount: vec![],
        locked_balance: vec![],
        active_flag: true,
    }.save(deps.storage)?;

    DistributionConfig {
        denom: msg.distribution_denom.to_cw20(deps.api),
        amounts: map_u128(msg.distribution_amounts),
    }.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

// TODO: governance 용과 admin 용을 나누는게 좋을까
pub fn update_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
) -> ContractResult<Response> {
    // Validate, Execute

    // governance == admin 고려
    let mut authorized = false;
    let contract_config = ContractConfig::load(deps.storage)?;

    let mut campaign_info = CampaignInfo::load(deps.storage)?;

    if contract_config.is_governance(&info.sender) {
        authorized = true;

        if url.is_some() {
            campaign_info.url = url.unwrap();
        }
    }

    if contract_config.is_admin(&info.sender) {
        authorized = true;

        if title.is_some() {
            campaign_info.title = title.unwrap();
        }

        if description.is_some() {
            campaign_info.description = description.unwrap();
        }
    }

    if !authorized {
        return Err(ContractError::Unauthorized {})
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
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut distribution_config = DistributionConfig::load(deps.storage)?;

    distribution_config.denom = denom.to_cw20(deps.api);
    distribution_config.amounts = map_u128(amounts);

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
    _env: Env,
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
    let free_balance = campaign_balance - campaign_state.locked_balance(denom.to_cw20(deps.api));
    let withdraw_amount = amount.map_or_else(
        || free_balance,
        |v| v.u128(),
    );

    if withdraw_amount > free_balance {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }

    // Execute
    let valkyrie_config: ValkyrieConfigResponse = deps.querier.query_wasm_smart(
        contract_config.governance,
        &valkyrie::governance::query_msgs::QueryMsg::ValkyrieConfig {},
    )?;
    let (burn_amount, receive_amount) = calc_ratio_amount(
        withdraw_amount,
        valkyrie_config.reward_withdraw_burn_rate,
    );

    let denom_cw20 = denom.to_cw20(deps.api);
    let burn_msg = make_send_msg(
        denom_cw20.clone(),
        burn_amount,
        &Addr::unchecked(valkyrie_config.burn_contract), //valkyrie_config 에 저장할 때 유효성 검사하므로 여기서는 하지 않음.
    );
    let send_msg = make_send_msg(
        denom_cw20,
        receive_amount,
        &info.sender,
    );

    // Response
    Ok(
        Response {
            submessages: vec![],
            messages: vec![burn_msg, send_msg],
            attributes: vec![],
            data: None,
        }
    )
}

pub fn claim_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // Execute
    let mut messages: Vec<CosmosMsg> = vec![];

    let mut campaign_state = CampaignState::load(deps.storage)?;
    let mut participation = Participation::load(deps.storage, &info.sender)?;

    for (denom, amount) in participation.rewards {
        campaign_state.unlock_balance(denom.clone(), amount)?;
        messages.push(make_send_msg(denom, amount, &info.sender));
    }

    participation.rewards = vec![];

    participation.save(deps.storage)?;

    // Response
    Ok(
        Response {
            submessages: vec![],
            messages,
            attributes: vec![],
            data: None,
        }
    )
}

pub fn participate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    // Validate
    let mut campaign_state = CampaignState::load(deps.storage)?;
    if !campaign_state.active_flag {
        return Err(ContractError::Std(StdError::generic_err("Deactivated campaign")))
    }

    if Participation::load(deps.storage, &info.sender).is_ok() {
        return Err(ContractError::Std(StdError::generic_err("Already participated")));
    }

    // Execute
    let distribution_config = DistributionConfig::load(deps.storage)?;

    let mut referrer = if referrer.is_some() {
        referrer.unwrap().to_address(deps.api).ok() // Ignore wrong referrer
    } else {
        None
    };
    let my_participation = Participation {
        actor_address: info.sender,
        referrer_address: referrer.clone(),
        rewards: vec![],
    };

    let mut participations = vec![my_participation];

    let mut remain_distance = distribution_config.amounts.len() - 1;
    while referrer.is_some() && remain_distance > 0 {
        let participation = Participation::load(deps.storage, &referrer.unwrap())?;
        referrer = participation.referrer_address.clone();
        participations.push(participation);
        remain_distance -= 1;
    }

    let mut distributions: Vec<(Addr, Vec<(Denom, u128)>)> = vec![];

    for (participation, reward_amount) in participations.iter_mut().zip(distribution_config.amounts) {
        participation.plus_reward(distribution_config.denom.clone(), reward_amount);
        participation.save(deps.storage)?;

        campaign_state.plus_distribution(distribution_config.denom.clone(), reward_amount);

        distributions.push((participation.actor_address.clone(), vec![(distribution_config.denom.clone(), reward_amount)]))
    }

    campaign_state.participation_count += 1;
    campaign_state.save(deps.storage)?;

    let campaign_balance = query_balance(
        &deps.querier,
        deps.api,
        distribution_config.denom.clone(),
        env.contract.address,
    )?;

    if campaign_state.locked_balance(distribution_config.denom) > campaign_balance {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }


    //TODO: boost msg

    // Response
    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![],
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
    denom: Denom,
    amount_with_tax: u128,
    recipient: &Addr,
) -> CosmosMsg {
    match denom {
        Denom::Native(denom) => message_factories::native_send(
            denom,
            recipient,
            Uint128::from(amount_with_tax),
        ),
        Denom::Cw20(contract_address) => message_factories::cw20_transfer(
            &contract_address,
            recipient,
            Uint128::from(amount_with_tax),
        ),
    }
}