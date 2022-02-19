use cosmwasm_std::{Addr, Binary, CosmosMsg, Env, MessageInfo, ReplyOn, Response, SubMsg, to_binary, Uint128, WasmMsg};

use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{DEFAULT_SENDER, default_sender, VALKYRIE_TICKET_TOKEN, VALKYRIE_TOKEN};
use valkyrie::test_constants::campaign::{CAMPAIGN_DESCRIPTION, CAMPAIGN_PARAMETER_KEY, CAMPAIGN_TITLE, CAMPAIGN_URL, PARTICIPATION_REWARD_AMOUNT, PARTICIPATION_REWARD_DENOM_NATIVE, REFERRAL_REWARD_AMOUNTS, DEPOSIT_DENOM_NATIVE, DEPOSIT_AMOUNT, DEPOSIT_LOCK_PERIOD, PARTICIPATION_REWARD_LOCK_PERIOD, REFERRAL_REWARD_LOCK_PERIOD};
use valkyrie::test_constants::campaign_manager::{CAMPAIGN_CODE_ID, CAMPAIGN_MANAGER, campaign_manager_env};
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
    vp_burn_amount: Option<Uint128>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
) -> ContractResult<Response> {
    create_campaign(
        deps.as_mut(),
        env,
        info,
        config_msg,
        deposit_denom,
        deposit_amount,
        deposit_lock_period,
        vp_burn_amount,
        qualifier,
        qualification_description,
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
        participation_reward_lock_period: PARTICIPATION_REWARD_LOCK_PERIOD,
        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
        referral_reward_lock_period: REFERRAL_REWARD_LOCK_PERIOD,
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
        None,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let (_, info, response) = default(&mut deps);

    assert_eq!(response.messages, vec![
        SubMsg {
            id: REPLY_CREATE_CAMPAIGN,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: CAMPAIGN_CODE_ID,
                msg: to_binary(&CampaignInstantiateMsg {
                    governance: GOVERNANCE.to_string(),
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
                        participation_reward_lock_period: PARTICIPATION_REWARD_LOCK_PERIOD,
                        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
                        referral_reward_lock_period: REFERRAL_REWARD_LOCK_PERIOD,
                    }).unwrap(),
                    deposit_denom: Some(Denom::Native(DEPOSIT_DENOM_NATIVE.to_string())),
                    deposit_amount: DEPOSIT_AMOUNT,
                    deposit_lock_period: DEPOSIT_LOCK_PERIOD,
                    vp_token: VALKYRIE_TICKET_TOKEN.to_string(),
                    vp_burn_amount: Uint128::zero(),
                    qualifier: None,
                    qualification_description: None,
                    referral_reward_token: VALKYRIE_TOKEN.to_string(),
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
