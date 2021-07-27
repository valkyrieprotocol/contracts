use cosmwasm_std::{CosmosMsg, Env, MessageInfo, ReplyOn, Response, to_binary, Uint128, WasmMsg, Addr, Binary, SubMsg};
use cw20::Cw20ExecuteMsg;

use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use valkyrie::common::{ContractResult, Denom, ExecutionMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, DEFAULT_SENDER, expect_generic_err, default_sender};

use crate::executions::{create_campaign, REPLY_CREATE_CAMPAIGN};
use crate::states::CreateCampaignContext;
use crate::tests::{CAMPAIGN_CODE_ID, CREATION_FEE_AMOUNT, GOVERNANCE, TOKEN_CONTRACT, FUND_MANAGER};
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_info};

pub const CAMPAIGN_TITLE: &str = "CampaignTitle";
pub const CAMPAIGN_DESCRIPTION: &str = "CampaignDescription";
pub const CAMPAIGN_URL: &str = "https://campaign.url";
pub const PARAMETER_KEY: &str = "vkr";
pub const DISTRIBUTION_TOKEN: &str = "uusd";
pub const DISTRIBUTION_AMOUNTS: [Uint128; 3] = [Uint128::new(100), Uint128::new(80), Uint128::new(20)];

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
    config_msg: Binary,
    proxies: Vec<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    create_campaign(
        deps.as_mut(),
        env,
        info,
        sender,
        amount,
        config_msg,
        proxies,
        executions,
    )
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(TOKEN_CONTRACT, &[]);

    let campaign_config_msg = CampaignConfigMsg {
        title: CAMPAIGN_TITLE.to_string(),
        description: CAMPAIGN_DESCRIPTION.to_string(),
        url: CAMPAIGN_URL.to_string(),
        parameter_key: PARAMETER_KEY.to_string(),
        distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
        distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec()
    };

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT,
        to_binary(&campaign_config_msg).unwrap(),
        vec![],
        vec![],
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

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
                    campaign_manager: MOCK_CONTRACT_ADDR.to_string(),
                    fund_manager: FUND_MANAGER.to_string(),
                    admin: DEFAULT_SENDER.to_string(),
                    creator: DEFAULT_SENDER.to_string(),
                    proxies: vec![],
                    config_msg: to_binary(&CampaignConfigMsg {
                        title: CAMPAIGN_TITLE.to_string(),
                        description: CAMPAIGN_DESCRIPTION.to_string(),
                        url: CAMPAIGN_URL.to_string(),
                        parameter_key: PARAMETER_KEY.to_string(),
                        distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
                        distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec(),
                    }).unwrap(),
                    executions: vec![],
                }).unwrap(),
                funds: vec![],
                label: String::new(),
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        },
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
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
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);
    super::update_campaign_config::will_success(
        &mut deps,
        None,
        Some(Uint128::zero()),
        None,
        None,
        None,
        None,
        None,
    );

    let response = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        DEFAULT_SENDER.to_string(),
        Uint128::zero(),
        to_binary(&CampaignConfigMsg {
            title: CAMPAIGN_TITLE.to_string(),
            description: CAMPAIGN_DESCRIPTION.to_string(),
            url: CAMPAIGN_URL.to_string(),
            parameter_key: PARAMETER_KEY.to_string(),
            distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
            distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec(),
        }).unwrap(),
        vec![],
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
                    campaign_manager: MOCK_CONTRACT_ADDR.to_string(),
                    fund_manager: FUND_MANAGER.to_string(),
                    admin: DEFAULT_SENDER.to_string(),
                    creator: DEFAULT_SENDER.to_string(),
                    proxies: vec![],
                    config_msg: to_binary(&CampaignConfigMsg {
                        title: CAMPAIGN_TITLE.to_string(),
                        description: CAMPAIGN_DESCRIPTION.to_string(),
                        url: CAMPAIGN_URL.to_string(),
                        parameter_key: PARAMETER_KEY.to_string(),
                        distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
                        distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec(),
                    }).unwrap(),
                    executions: vec![],
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
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(TOKEN_CONTRACT, &[]),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT.checked_sub(Uint128::new(1)).unwrap(),
        to_binary(&CampaignConfigMsg {
            title: CAMPAIGN_TITLE.to_string(),
            description: CAMPAIGN_DESCRIPTION.to_string(),
            url: CAMPAIGN_URL.to_string(),
            parameter_key: PARAMETER_KEY.to_string(),
            distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
            distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec(),
        }).unwrap(),
        vec![],
        vec![],
    );

    expect_generic_err(
        &result,
        format!("Insufficient creation fee (Fee = {})", CREATION_FEE_AMOUNT).as_str(),
    );
}

#[test]
fn failed_invalid_creation_token() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        DEFAULT_SENDER.to_string(),
        CREATION_FEE_AMOUNT,
        to_binary(&CampaignConfigMsg {
            title: CAMPAIGN_TITLE.to_string(),
            description: CAMPAIGN_DESCRIPTION.to_string(),
            url: CAMPAIGN_URL.to_string(),
            parameter_key: PARAMETER_KEY.to_string(),
            distribution_denom: Denom::Native(DISTRIBUTION_TOKEN.to_string()),
            distribution_amounts: DISTRIBUTION_AMOUNTS.to_vec(),
        }).unwrap(),
        vec![],
        vec![],
    );
    expect_generic_err(&result, "Invalid creation fee token");
}
