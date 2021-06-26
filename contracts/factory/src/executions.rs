use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128, StdError, SubMsg, ReplyOn, Reply, Decimal, to_binary};
use valkyrie::factory::execute_msgs::InstantiateMsg;
use valkyrie::common::ContractResult;
use crate::states::{FactoryConfig, is_governance, CreateCampaignContext, Campaign, CampaignConfig};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::find;
use valkyrie::campaign::enumerations::Denom;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    FactoryConfig {
        governance: deps.api.addr_validate(msg.governance.as_str())?,
        token_contract: deps.api.addr_validate(msg.token_contract.as_str())?,
        distributor: deps.api.addr_validate(msg.distributor.as_str())?,
        campaign_code_id: msg.campaign_code_id,
        creation_fee_amount: msg.creation_fee_amount,
    }.save(deps.storage)?;

    CampaignConfig {
        reward_withdraw_burn_rate: msg.reward_withdraw_burn_rate,
        campaign_deactivate_period: msg.campaign_deactivate_period,
    }.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_factory_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign_code_id: Option<u64>,
    creation_fee_amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut factory_config = FactoryConfig::load(deps.storage)?;

    if campaign_code_id.is_some() {
        factory_config.campaign_code_id = campaign_code_id.unwrap();
    }

    if creation_fee_amount.is_some() {
        factory_config.creation_fee_amount = creation_fee_amount.unwrap();
    }

    factory_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn update_campaign_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    reward_withdraw_burn_rate: Option<Decimal>,
    campaign_deactivate_period: Option<u64>,
) -> ContractResult<Response> {
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut campaign_config = CampaignConfig::load(deps.storage)?;

    if let Some(reward_withdraw_burn_rate) = reward_withdraw_burn_rate {
        campaign_config.reward_withdraw_burn_rate = reward_withdraw_burn_rate;
    }

    if let Some(campaign_deactivate_period) = campaign_deactivate_period {
        campaign_config.campaign_deactivate_period = campaign_deactivate_period;
    }

    campaign_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub const REPLY_CREATE_CAMPAIGN: u64 = 1;

pub fn create_campaign(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
    title: String,
    url: String,
    description: String,
    parameter_key: String,
    distribution_denom: Denom,
    distribution_amounts: Vec<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let factory_config = FactoryConfig::load(deps.storage)?;

    if amount < factory_config.creation_fee_amount {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Insufficient creation fee (Fee = {})", factory_config.creation_fee_amount),
        )));
    }

    // Execute
    CreateCampaignContext {
        code_id: factory_config.campaign_code_id,
        creator: deps.api.addr_validate(sender.as_str())?,
    }.save(deps.storage)?;

    //TODO: CW20 과 같은 형태로 변경
    let create_campaign_msg = message_factories::wasm_instantiate(
        factory_config.campaign_code_id,
        Some(factory_config.governance.clone()),
        to_binary(&valkyrie::campaign::execute_msgs::InstantiateMsg {
            governance: factory_config.governance.to_string(),
            distributor: factory_config.distributor.to_string(),
            token_contract: factory_config.token_contract.to_string(),
            factory: env.contract.address.to_string(),
            title,
            url,
            description,
            parameter_key,
            distribution_denom,
            distribution_amounts,
        })?,
    );

    //TODO: 별도의 msg 를 함께 보낼 필요는 없으려나?

    //TODO: 수수료 받은게 바로 반영이 안되서 transfer 를 못하는 것 같음
    //TODO: InvalidArgument desc = failed to execute message; message index: 0: Overflow: Cannot Sub with 0 and 100000000: execute wasm contract failed: invalid request
    let fee_send_msg = message_factories::cw20_transfer(
        &factory_config.token_contract,
        &factory_config.governance,
        amount,
    );

    // Response
    Ok(Response {
        submessages: vec![
            SubMsg {
                id: REPLY_CREATE_CAMPAIGN,
                msg: create_campaign_msg,
                gas_limit: None,
                reply_on: ReplyOn::Success, //TODO: Fail 이면 reply 가 호출되지 않더라도 트랜잭션은 실패하겠지?
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
        created_block: env.block.height,
    }.save(deps.storage)?;

    CreateCampaignContext::clear(deps.storage);

    Ok(Response::default())
}