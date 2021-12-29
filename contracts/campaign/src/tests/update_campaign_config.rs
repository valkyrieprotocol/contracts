use cosmwasm_std::{Env, MessageInfo, Response, Addr, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::executions::{update_campaign_config, MIN_TITLE_LENGTH, MAX_TITLE_LENGTH, MIN_DESC_LENGTH, MAX_DESC_LENGTH, MIN_URL_LENGTH, MAX_URL_LENGTH, MIN_PARAM_KEY_LENGTH, MAX_PARAM_KEY_LENGTH};
use crate::states::CampaignConfig;
use valkyrie::test_constants::campaign::{CAMPAIGN_ADMIN, campaign_admin_sender, campaign_env};
use valkyrie::test_constants::default_sender;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    deposit_amount: Option<Uint128>,
    deposit_lock_period: Option<u64>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    admin: Option<String>,
) -> ContractResult<Response> {
    update_campaign_config(
        deps.as_mut(),
        env,
        info,
        title,
        description,
        url,
        parameter_key,
        deposit_amount,
        deposit_lock_period,
        qualifier,
        qualification_description,
        admin,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    deposit_amount: Option<Uint128>,
    deposit_lock_period: Option<u64>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    admin: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        title,
        description,
        url,
        parameter_key,
        deposit_amount,
        deposit_lock_period,
        qualifier,
        qualification_description,
        admin,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let title = "Title2".to_string();
    let description = "Desc2".to_string();
    let url = "https://url2.url".to_string();
    let parameter_key = "vkr2".to_string();
    let deposit_amount = Uint128::new(99);
    let deposit_lock_period = 199u64;
    let qualifier = "Qualifier2".to_string();
    let qualification_description = "QualificationDescription2".to_string();
    let admin = "Admin2".to_string();

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        Some(url.clone()),
        Some(parameter_key.clone()),
        Some(deposit_amount),
        Some(deposit_lock_period),
        Some(qualifier.clone()),
        Some(qualification_description.clone()),
        Some(admin.clone()),
    );

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config.title, title);
    assert_eq!(campaign_config.description, description);
    assert_eq!(campaign_config.url, url);
    assert_eq!(campaign_config.parameter_key, parameter_key);
    assert_eq!(campaign_config.deposit_amount, deposit_amount);
    assert_eq!(campaign_config.deposit_lock_period, deposit_lock_period);
    assert_eq!(campaign_config.qualifier, Some(Addr::unchecked(qualifier)));
    assert_eq!(campaign_config.admin, Addr::unchecked(CAMPAIGN_ADMIN));

    let admin_nominee = CampaignConfig::may_load_admin_nominee(&deps.storage).unwrap();
    assert_eq!(admin_nominee, Some(Addr::unchecked(admin.as_str())));
}

#[test]
fn succeed_update_info_after_activation() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let title = "Title2".to_string();
    let description = "Desc2".to_string();
    let deposit_amount = Uint128::new(99);
    let deposit_lock_period = 199u64;
    let qualifier = "Qualifier2".to_string();
    let qualification_description = "QualificationDescription2".to_string();
    let admin = "Admin2".to_string();

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        None,
        None,
        Some(deposit_amount),
        Some(deposit_lock_period),
        Some(qualifier.clone()),
        Some(qualification_description.clone()),
        Some(admin.clone()),
    );

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config.title, title);
    assert_eq!(campaign_config.description, description);
    assert_eq!(campaign_config.deposit_amount, deposit_amount);
    assert_eq!(campaign_config.deposit_lock_period, deposit_lock_period);
    assert_eq!(campaign_config.qualifier, Some(Addr::unchecked(qualifier)));
    assert_eq!(campaign_config.qualification_description, Some(qualification_description));
    assert_eq!(campaign_config.admin, Addr::unchecked(CAMPAIGN_ADMIN));

    let admin_nominee = CampaignConfig::may_load_admin_nominee(&deps.storage).unwrap();
    assert_eq!(admin_nominee, Some(Addr::unchecked(admin.as_str())));
}

#[test]
fn failed_update_url_after_activation() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        Some("https://url2.url".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    expect_generic_err(&result, "Only modifiable in pending status");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        None,
        Some("vkr2".to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    expect_generic_err(&result, "Only modifiable in pending status");
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_invalid_title() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        Some(std::iter::repeat('b').take(MIN_TITLE_LENGTH - 1).collect()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "Title too short");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        Some(std::iter::repeat('b').take(MAX_TITLE_LENGTH + 1).collect()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "Title too long");
}

#[test]
fn failed_invalid_description() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        Some(std::iter::repeat('b').take(MIN_DESC_LENGTH - 1).collect()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "Description too short");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        Some(std::iter::repeat('b').take(MAX_DESC_LENGTH + 1).collect()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "Description too long");
}

#[test]
fn failed_invalid_url() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        Some(std::iter::repeat('b').take(MIN_URL_LENGTH - 1).collect()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "Url too short");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        Some(std::iter::repeat('b').take(MAX_URL_LENGTH + 1).collect()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "Url too long");
}

#[test]
fn failed_invalid_parameter_key() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        None,
        Some(std::iter::repeat('b').take(MIN_PARAM_KEY_LENGTH - 1).collect()),
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "ParameterKey too short");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        None,
        Some(std::iter::repeat('b').take(MAX_PARAM_KEY_LENGTH + 1).collect()),
        None,
        None,
        None,
        None,
        None,
    );
    expect_generic_err(&result, "ParameterKey too long");
}
