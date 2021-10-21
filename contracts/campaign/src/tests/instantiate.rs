use cosmwasm_std::{Addr, Env, MessageInfo, Response, to_binary, Uint128};
use cosmwasm_std::testing::mock_env;

use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;
use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::*;
use valkyrie::test_constants::campaign_manager::{CAMPAIGN_MANAGER, campaign_manager_sender};
use valkyrie::test_constants::governance::GOVERNANCE;
use valkyrie::test_utils::expect_generic_err;

use crate::executions::*;
use crate::states::{CampaignConfig, CampaignState, RewardConfig};
use valkyrie::test_constants::VALKYRIE_TOKEN;

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
    participation_reward_denom: Denom,
    participation_reward_amount: Uint128,
    participation_reward_lock_period: u64,
    referral_reward_amounts: Vec<Uint128>,
    referral_reward_lock_period: u64,
) -> ContractResult<Response> {
    let config_msg = CampaignConfigMsg {
        title,
        url,
        description,
        parameter_key,
        participation_reward_denom,
        participation_reward_amount,
        participation_reward_lock_period,
        referral_reward_amounts,
        referral_reward_lock_period,
    };

    let msg = CampaignInstantiateMsg {
        governance: GOVERNANCE.to_string(),
        campaign_manager: CAMPAIGN_MANAGER.to_string(),
        deposit_denom: Some(Denom::Native(DEPOSIT_DENOM_NATIVE.to_string())),
        deposit_amount: DEPOSIT_AMOUNT,
        deposit_lock_period: DEPOSIT_LOCK_PERIOD,
        qualifier,
        qualification_description,
        admin: CAMPAIGN_ADMIN.to_string(),
        creator: CAMPAIGN_ADMIN.to_string(),
        referral_reward_token: VALKYRIE_TOKEN.to_string(),
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
    participation_reward_denom: Denom,
    participation_reward_amount: Uint128,
    participation_reward_lock_period: u64,
    referral_reward_amounts: Vec<Uint128>,
    referral_reward_lock_period: u64,
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
        participation_reward_denom,
        participation_reward_amount,
        participation_reward_lock_period,
        referral_reward_amounts,
        referral_reward_lock_period,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        title: CAMPAIGN_TITLE.to_string(),
        description: CAMPAIGN_DESCRIPTION.to_string(),
        url: CAMPAIGN_URL.to_string(),
        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
        deposit_denom: Some(cw20::Denom::Native(DEPOSIT_DENOM_NATIVE.to_string())),
        deposit_amount: DEPOSIT_AMOUNT,
        deposit_lock_period: DEPOSIT_LOCK_PERIOD,
        qualifier: None,
        qualification_description: None,
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
        deposit_amount: Uint128::zero(),
        active_flag: false,
        last_active_height: None,
        chain_id: env.block.chain_id,
    });

    let distribution_config = RewardConfig::load(&deps.storage).unwrap();
    assert_eq!(distribution_config, RewardConfig {
        participation_reward_denom: cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_lock_period: PARTICIPATION_REWARD_LOCK_PERIOD,
        referral_reward_token: Addr::unchecked(VALKYRIE_TOKEN),
        referral_reward_amounts: REFERRAL_REWARD_AMOUNTS.to_vec(),
        referral_reward_lock_period: REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
    );
    expect_generic_err(&result, "ParameterKey too long");
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
    );

    will_success(
        &mut deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        None,
        None,
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        Uint128::zero(),
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        vec![],
        REFERRAL_REWARD_LOCK_PERIOD,
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
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        vec![Uint128::zero(), Uint128::zero()],
        REFERRAL_REWARD_LOCK_PERIOD,
    );
    expect_generic_err(&result, "Invalid reward scheme");
}
