use cosmwasm_std::{Addr, Binary, CosmosMsg, Env, MessageInfo, ReplyOn, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::common::{ContractResult, Denom, ExecutionMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{DEFAULT_SENDER, default_sender};
use valkyrie::test_constants::campaign::{CAMPAIGN_DESCRIPTION, CAMPAIGN_PARAMETER_KEY, CAMPAIGN_TITLE, CAMPAIGN_URL, PARTICIPATION_REWARD_AMOUNT, PARTICIPATION_REWARD_DENOM_NATIVE, REFERRAL_REWARD_AMOUNTS};
use valkyrie::test_constants::campaign_manager::{CAMPAIGN_CODE_ID, CAMPAIGN_MANAGER, campaign_manager_env, CREATION_FEE_AMOUNT, REFERRAL_REWARD_TOKEN, CREATION_FEE_TOKEN, creation_fee_token};
use valkyrie::test_constants::fund_manager::FUND_MANAGER;
use valkyrie::test_constants::governance::GOVERNANCE;
use valkyrie::test_utils::expect_generic_err;

use crate::executions::{create_campaign, REPLY_CREATE_CAMPAIGN};
use crate::states::CreateCampaignContext;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
    config_msg: Binary,
    ticket_count: u64,
    qualifier: Option<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    create_campaign(
        deps.as_mut(),
        env,
        info,
        sender,
        amount,
        config_msg,
        ticket_count,
        qualifier,
        executions,
    )
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = creation_fee_token();

    let campaign_config_msg = CampaignConfigMsg {
        title: CAMPAIGN_TITLE.to_string(),
        description: CAMPAIGN_DESCRIPTION.to_string(),
        url: CAMPAIGN_URL.to_string(),
        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
        participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
    };

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT,
        to_binary(&campaign_config_msg).unwrap(),
        1,
        None,
        vec![],
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let (_, _, response) = default(&mut deps);

    assert_eq!(response.messages, vec![
        SubMsg {
            id: REPLY_CREATE_CAMPAIGN,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(GOVERNANCE.to_string()),
                code_id: CAMPAIGN_CODE_ID,
                msg: to_binary(&CampaignInstantiateMsg {
                    governance: GOVERNANCE.to_string(),
                    fund_manager: FUND_MANAGER.to_string(),
                    campaign_manager: CAMPAIGN_MANAGER.to_string(),
                    admin: DEFAULT_SENDER.to_string(),
                    creator: DEFAULT_SENDER.to_string(),
                    config_msg: to_binary(&CampaignConfigMsg {
                        title: CAMPAIGN_TITLE.to_string(),
                        description: CAMPAIGN_DESCRIPTION.to_string(),
                        url: CAMPAIGN_URL.to_string(),
                        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
                        participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
                        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
                        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
                    }).unwrap(),
                    ticket_amount: 1,
                    qualifier: None,
                    executions: vec![],
                    referral_reward_token: REFERRAL_REWARD_TOKEN.to_string(),
                }).unwrap(),
                funds: vec![],
                label: String::new(),
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        },
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: CREATION_FEE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: FUND_MANAGER.to_string(),
                amount: CREATION_FEE_AMOUNT,
            }).unwrap(),
        })),
    ]);

    let context = CreateCampaignContext::load(&deps.storage).unwrap();
    assert_eq!(context, CreateCampaignContext {
        code_id: CAMPAIGN_CODE_ID,
        creator: Addr::unchecked(DEFAULT_SENDER),
    });
}

#[test]
fn succeed_zero_creation_fee() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_config::will_success(
        &mut deps,
        None,
        None,
        None,
        None,
        Some(Uint128::zero()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let response = exec(
        &mut deps,
        campaign_manager_env(),
        creation_fee_token(),
        DEFAULT_SENDER.to_string(),
        Uint128::zero(),
        to_binary(&CampaignConfigMsg {
            title: CAMPAIGN_TITLE.to_string(),
            description: CAMPAIGN_DESCRIPTION.to_string(),
            url: CAMPAIGN_URL.to_string(),
            parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
            participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
            participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
            referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
        }).unwrap(),
        1,
        None,
        vec![],
    ).unwrap();

    assert_eq!(response.messages, vec![
        SubMsg {
            id: REPLY_CREATE_CAMPAIGN,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(GOVERNANCE.to_string()),
                code_id: CAMPAIGN_CODE_ID,
                msg: to_binary(&CampaignInstantiateMsg {
                    governance: GOVERNANCE.to_string(),
                    campaign_manager: CAMPAIGN_MANAGER.to_string(),
                    fund_manager: FUND_MANAGER.to_string(),
                    admin: DEFAULT_SENDER.to_string(),
                    creator: DEFAULT_SENDER.to_string(),
                    config_msg: to_binary(&CampaignConfigMsg {
                        title: CAMPAIGN_TITLE.to_string(),
                        description: CAMPAIGN_DESCRIPTION.to_string(),
                        url: CAMPAIGN_URL.to_string(),
                        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
                        participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
                        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
                        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
                    }).unwrap(),
                    ticket_amount: 1,
                    qualifier: None,
                    executions: vec![],
                    referral_reward_token: REFERRAL_REWARD_TOKEN.to_string(),
                }).unwrap(),
                funds: vec![],
                label: String::new(),
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        },
    ]);

    let context = CreateCampaignContext::load(&deps.storage).unwrap();
    assert_eq!(context, CreateCampaignContext {
        code_id: CAMPAIGN_CODE_ID,
        creator: Addr::unchecked(DEFAULT_SENDER),
    });
}

#[test]
fn failed_insufficient_creation_fee() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        creation_fee_token(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT.checked_sub(Uint128::new(1)).unwrap(),
        to_binary(&CampaignConfigMsg {
            title: CAMPAIGN_TITLE.to_string(),
            description: CAMPAIGN_DESCRIPTION.to_string(),
            url: CAMPAIGN_URL.to_string(),
            parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
            participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
            participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
            referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
        }).unwrap(),
        1,
        None,
        vec![],
    );

    expect_generic_err(
        &result,
        format!("Insufficient creation fee (Fee = {})", CREATION_FEE_AMOUNT).as_str(),
    );
}

#[test]
fn failed_invalid_creation_token() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT,
        to_binary(&CampaignConfigMsg {
            title: CAMPAIGN_TITLE.to_string(),
            description: CAMPAIGN_DESCRIPTION.to_string(),
            url: CAMPAIGN_URL.to_string(),
            parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
            participation_reward_denom: Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
            participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
            referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
        }).unwrap(),
        1,
        None,
        vec![],
    );
    expect_generic_err(&result, "Invalid creation fee token");
}
