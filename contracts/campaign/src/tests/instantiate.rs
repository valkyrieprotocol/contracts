use cosmwasm_std::{Addr, Env, MessageInfo, Response, to_binary, Uint128};
use cosmwasm_std::testing::mock_env;

use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::common::{ContractResult, Denom, Execution, ExecutionMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::*;
use valkyrie::test_constants::campaign_manager::{CAMPAIGN_MANAGER, campaign_manager_sender, REFERRAL_REWARD_TOKEN};
use valkyrie::test_constants::fund_manager::FUND_MANAGER;
use valkyrie::test_constants::governance::GOVERNANCE;
use valkyrie::test_utils::expect_generic_err;

use crate::executions::{instantiate, MAX_DESC_LENGTH, MAX_PARAM_KEY_LENGTH, MAX_TITLE_LENGTH, MAX_URL_LENGTH, MIN_DESC_LENGTH, MIN_PARAM_KEY_LENGTH, MIN_TITLE_LENGTH, MIN_URL_LENGTH};
use crate::states::{CampaignConfig, CampaignState, RewardConfig};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    url: String,
    parameter_key: String,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    executions: Vec<ExecutionMsg>,
    participation_reward_denom: Denom,
    participation_reward_amount: Uint128,
    referral_reward_amounts: Vec<Uint128>,
) -> ContractResult<Response> {
    let config_msg = CampaignConfigMsg {
        title,
        url,
        description,
        parameter_key,
        participation_reward_denom,
        participation_reward_amount,
        referral_reward_amounts,
    };

    let msg = CampaignInstantiateMsg {
        governance: GOVERNANCE.to_string(),
        campaign_manager: CAMPAIGN_MANAGER.to_string(),
        fund_manager: FUND_MANAGER.to_string(),
        collateral_denom: Some(Denom::Native(COLLATERAL_DENOM_NATIVE.to_string())),
        collateral_amount: COLLATERAL_AMOUNT,
        collateral_lock_period: COLLATERAL_LOCK_PERIOD,
        qualifier,
        qualification_description,
        executions,
        admin: CAMPAIGN_ADMIN.to_string(),
        creator: CAMPAIGN_ADMIN.to_string(),
        referral_reward_token: REFERRAL_REWARD_TOKEN.to_string(),
        config_msg: to_binary(&config_msg)?,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn will_success(
    deps: &mut CustomDeps,
    title: String,
    description: String,
    url: String,
    parameter_key: String,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    executions: Vec<ExecutionMsg>,
    participation_reward_denom: Denom,
    participation_reward_amount: Uint128,
    referral_reward_amounts: Vec<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_manager_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        title,
        description,
        url,
        parameter_key,
        qualifier,
        qualification_description,
        executions,
        participation_reward_denom,
        participation_reward_amount,
        referral_reward_amounts,
    ).unwrap();

    (env, info, response)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    will_success(
        deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    )
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    let (env, _, _) = default(&mut deps);

    let campaign_info = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_info, CampaignConfig {
        governance: Addr::unchecked(GOVERNANCE),
        campaign_manager: Addr::unchecked(CAMPAIGN_MANAGER),
        fund_manager: Addr::unchecked(FUND_MANAGER),
        title: CAMPAIGN_TITLE.to_string(),
        description: CAMPAIGN_DESCRIPTION.to_string(),
        url: CAMPAIGN_URL.to_string(),
        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
        collateral_denom: Some(cw20::Denom::Native(COLLATERAL_DENOM_NATIVE.to_string())),
        collateral_amount: COLLATERAL_AMOUNT,
        collateral_lock_period: COLLATERAL_LOCK_PERIOD,
        qualifier: None,
        qualification_description: None,
        executions: vec![],
        admin: Addr::unchecked(CAMPAIGN_ADMIN),
        creator: Addr::unchecked(CAMPAIGN_ADMIN),
        created_at: env.block.time,
    });

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state, CampaignState {
        actor_count: 0,
        participation_count: 0,
        cumulative_participation_reward_amount: Uint128::zero(),
        cumulative_referral_reward_amount: Uint128::zero(),
        locked_balances: vec![],
        balances: vec![],
        collateral_amount: Uint128::zero(),
        active_flag: false,
        last_active_height: None,
        chain_id: env.block.chain_id,
    });

    let distribution_config = RewardConfig::load(&deps.storage).unwrap();
    assert_eq!(distribution_config, RewardConfig {
        participation_reward_denom: cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        referral_reward_token: Addr::unchecked(REFERRAL_REWARD_TOKEN),
        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
    });
}

#[test]
fn failed_invalid_title() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        std::iter::repeat('a').take(MIN_TITLE_LENGTH - 1).collect(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "Title too short");

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        std::iter::repeat('a').take(MAX_TITLE_LENGTH + 1).collect(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "Title too long");
}

#[test]
fn failed_invalid_description() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        std::iter::repeat('a').take(MIN_DESC_LENGTH - 1).collect(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "Description too short");

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        std::iter::repeat('a').take(MAX_DESC_LENGTH + 1).collect(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "Description too long");
}

#[test]
fn failed_invalid_url() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        std::iter::repeat('a').take(MIN_URL_LENGTH - 1).collect(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "Url too short");

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        std::iter::repeat('a').take(MAX_URL_LENGTH + 1).collect(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "Url too long");
}

#[test]
fn failed_invalid_parameter_key() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        std::iter::repeat('a').take(MIN_PARAM_KEY_LENGTH - 1).collect(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "ParameterKey too short");

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        std::iter::repeat('a').take(MAX_PARAM_KEY_LENGTH + 1).collect(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );
    expect_generic_err(&result, "ParameterKey too long");
}

#[test]
fn test_execution_order() {
    let mut deps = custom_deps();

    let executions = vec![
        ExecutionMsg {
            order: 2,
            contract: "Contract1".to_string(),
            msg: to_binary("").unwrap(),
        },
        ExecutionMsg {
            order: 1,
            contract: "Contract1".to_string(),
            msg: to_binary("").unwrap(),
        },
        ExecutionMsg {
            order: 3,
            contract: "Contract1".to_string(),
            msg: to_binary("").unwrap(),
        },
    ];

    will_success(
        &mut deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        executions,
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
    );

    let campaign = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign.executions, vec![
        Execution {
            order: 1,
            contract: Addr::unchecked("Contract1"),
            msg: to_binary("").unwrap(),
        },
        Execution {
            order: 2,
            contract: Addr::unchecked("Contract1"),
            msg: to_binary("").unwrap(),
        },
        Execution {
            order: 3,
            contract: Addr::unchecked("Contract1"),
            msg: to_binary("").unwrap(),
        },
    ]);
}

#[test]
fn failed_invalid_amounts() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(
        &mut deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        vec![Uint128::zero(), Uint128::new(100)],
    );

    will_success(
        &mut deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        Uint128::zero(),
        vec![Uint128::zero(), Uint128::new(100)],
    );

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        vec![],
    );
    expect_generic_err(&result, "Invalid reward scheme");

    let result = exec(
        &mut deps,
        mock_env(),
        campaign_manager_sender(),
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        vec![],
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        vec![Uint128::zero(), Uint128::zero()],
    );
    expect_generic_err(&result, "Invalid reward scheme");
}
