use cosmwasm_std::{Addr, Binary, CosmosMsg, Env, MessageInfo, ReplyOn, Response, SubMsg, to_binary, Uint128, WasmMsg};

use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::common::{ContractResult, Denom, ExecutionMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{DEFAULT_SENDER, default_sender};
use valkyrie::test_constants::campaign::{CAMPAIGN_DESCRIPTION, CAMPAIGN_PARAMETER_KEY, CAMPAIGN_TITLE, CAMPAIGN_URL, PARTICIPATION_REWARD_AMOUNT, PARTICIPATION_REWARD_DENOM_NATIVE, REFERRAL_REWARD_AMOUNTS, DEPOSIT_DENOM_NATIVE, DEPOSIT_AMOUNT, DEPOSIT_LOCK_PERIOD};
use valkyrie::test_constants::campaign_manager::{CAMPAIGN_CODE_ID, CAMPAIGN_MANAGER, campaign_manager_env, REFERRAL_REWARD_TOKEN};
use valkyrie::test_constants::fund_manager::FUND_MANAGER;
use valkyrie::test_constants::governance::GOVERNANCE;

use crate::executions::{create_campaign, REPLY_CREATE_CAMPAIGN};
use crate::states::CreateCampaignContext;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    config_msg: Binary,
    deposit_denom: Option<Denom>,
    deposit_amount: Option<Uint128>,
    deposit_lock_period: Option<u64>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    create_campaign(
        deps.as_mut(),
        env,
        info,
        config_msg,
        deposit_denom,
        deposit_amount,
        deposit_lock_period,
        qualifier,
        qualification_description,
        executions,
    )
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = default_sender();

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
        to_binary(&campaign_config_msg).unwrap(),
        Some(Denom::Native(DEPOSIT_DENOM_NATIVE.to_string())),
        Some(DEPOSIT_AMOUNT),
        Some(DEPOSIT_LOCK_PERIOD),
        None,
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
                    deposit_denom: Some(Denom::Native(DEPOSIT_DENOM_NATIVE.to_string())),
                    deposit_amount: DEPOSIT_AMOUNT,
                    deposit_lock_period: DEPOSIT_LOCK_PERIOD,
                    qualifier: None,
                    qualification_description: None,
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
