use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128, Uint64, Binary, StdError, SubMsg, ReplyOn, Reply};
use valkyrie::factory::execute_msgs::InstantiateMsg;
use valkyrie::common::ContractResult;
use crate::states::{FactoryConfig, is_governance, CreateCampaignContext, Campaign};
use valkyrie::errors::ContractError;
use valkyrie::message_factories;
use valkyrie::utils::find;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    FactoryConfig {
        governance: deps.api.addr_validate(msg.governance.as_str())?,
        token_contract: deps.api.addr_validate(msg.token_contract.as_str())?,
        campaign_code_id: msg.campaign_code_id.u64(),
        creation_fee_amount: msg.creation_fee_amount.u128(),
    }.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    campaign_code_id: Option<Uint64>,
    creation_fee_amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    if !is_governance(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut factory_config = FactoryConfig::load(deps.storage)?;

    if campaign_code_id.is_some() {
        factory_config.campaign_code_id = campaign_code_id.unwrap().u64();
    }

    if creation_fee_amount.is_some() {
        factory_config.creation_fee_amount = creation_fee_amount.unwrap().u128();
    }

    factory_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub const REPLY_CREATE_CAMPAIGN: u64 = 1;

pub fn create_campaign(
    deps: DepsMut,
    _env: Env,
    sender: String,
    amount: Uint128,
    campaign_init_msg: Binary,
) -> ContractResult<Response> {
    // Validate
    let factory_config = FactoryConfig::load(deps.storage)?;

    if amount.u128() < factory_config.creation_fee_amount {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Insufficient creation fee (Fee = {})", factory_config.creation_fee_amount),
        )));
    }

    // Execute
    CreateCampaignContext {
        code_id: factory_config.campaign_code_id,
        creator: deps.api.addr_validate(sender.as_str())?,
    }.save(deps.storage)?;

    let create_campaign_msg = message_factories::wasm_instantiate(
        factory_config.campaign_code_id,
        Some(factory_config.governance.clone()),
        campaign_init_msg,
    );

    //TODO: 별도의 msg 를 함께 보낼 필요는 없으려나?
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