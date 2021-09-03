use cosmwasm_std::{Env, MessageInfo, Response, to_binary, Addr, Uint128};

use valkyrie::common::{ContractResult, ExecutionMsg, Execution};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::executions::{update_campaign_config, MIN_TITLE_LENGTH, MAX_TITLE_LENGTH, MIN_DESC_LENGTH, MAX_DESC_LENGTH, MIN_URL_LENGTH, MAX_URL_LENGTH, MIN_PARAM_KEY_LENGTH, MAX_PARAM_KEY_LENGTH};
use crate::states::CampaignConfig;
use valkyrie::test_constants::campaign::{campaign_admin_sender, campaign_env};
use valkyrie::test_constants::default_sender;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    collateral_amount: Option<Uint128>,
    collateral_lock_period: Option<u64>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    executions: Option<Vec<ExecutionMsg>>,
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
        collateral_amount,
        collateral_lock_period,
        qualifier,
        qualification_description,
        executions,
        admin,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    collateral_amount: Option<Uint128>,
    collateral_lock_period: Option<u64>,
    qualifier: Option<String>,
    qualification_description: Option<String>,
    executions: Option<Vec<ExecutionMsg>>,
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
        collateral_amount,
        collateral_lock_period,
        qualifier,
        qualification_description,
        executions,
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
    let collateral_amount = Uint128::new(99);
    let collateral_lock_period = 199u64;
    let qualifier = "Qualifier2".to_string();
    let qualification_description = "QualificationDescription2".to_string();
    let executions = vec![
        ExecutionMsg {
            order: 1,
            contract: "ExecContract".to_string(),
            msg: to_binary("").unwrap(),
        },
    ];
    let admin = "Admin2".to_string();

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        Some(url.clone()),
        Some(parameter_key.clone()),
        Some(collateral_amount),
        Some(collateral_lock_period),
        Some(qualifier.clone()),
        Some(qualification_description.clone()),
        Some(executions.clone()),
        Some(admin.clone()),
    );

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config.title, title);
    assert_eq!(campaign_config.description, description);
    assert_eq!(campaign_config.url, url);
    assert_eq!(campaign_config.parameter_key, parameter_key);
    assert_eq!(campaign_config.collateral_amount, collateral_amount);
    assert_eq!(campaign_config.collateral_lock_period, collateral_lock_period);
    assert_eq!(campaign_config.qualifier, Some(Addr::unchecked(qualifier)));
    assert_eq!(campaign_config.executions, executions.iter().map(|e| Execution {
        order: e.order,
        contract: Addr::unchecked(e.contract.as_str()),
        msg: e.msg.clone(),
    }).collect::<Vec<Execution>>());
    assert_eq!(campaign_config.admin, admin);
}

#[test]
fn succeed_update_info_after_activation() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let title = "Title2".to_string();
    let description = "Desc2".to_string();
    let collateral_amount = Uint128::new(99);
    let collateral_lock_period = 199u64;
    let qualifier = "Qualifier2".to_string();
    let qualification_description = "QualificationDescription2".to_string();
    let executions = vec![
        ExecutionMsg {
            order: 1,
            contract: "ExecContract".to_string(),
            msg: to_binary("").unwrap(),
        },
    ];
    let admin = "Admin2".to_string();

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        None,
        None,
        Some(collateral_amount),
        Some(collateral_lock_period),
        Some(qualifier.clone()),
        Some(qualification_description.clone()),
        Some(executions.clone()),
        Some(admin.clone()),
    );

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config.title, title);
    assert_eq!(campaign_config.description, description);
    assert_eq!(campaign_config.collateral_amount, collateral_amount);
    assert_eq!(campaign_config.collateral_lock_period, collateral_lock_period);
    assert_eq!(campaign_config.qualifier, Some(Addr::unchecked(qualifier)));
    assert_eq!(campaign_config.qualification_description, Some(qualification_description));
    assert_eq!(campaign_config.executions, executions.iter().map(|e| Execution {
        order: e.order,
        contract: Addr::unchecked(e.contract.as_str()),
        msg: e.msg.clone(),
    }).collect::<Vec<Execution>>());
    assert_eq!(campaign_config.admin, admin);
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
        None,
    );
    expect_generic_err(&result, "ParameterKey too long");
}

#[test]
fn test_execution_order() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let executions = vec![
        ExecutionMsg {
            order: 2,
            contract: "Contract2".to_string(),
            msg: to_binary("").unwrap(),
        },
        ExecutionMsg {
            order: 1,
            contract: "Contract2".to_string(),
            msg: to_binary("").unwrap(),
        },
        ExecutionMsg {
            order: 3,
            contract: "Contract2".to_string(),
            msg: to_binary("").unwrap(),
        },
    ];

    will_success(
        &mut deps,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(executions),
        None,
    );

    let campaign = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign.executions, vec![
        Execution {
            order: 1,
            contract: Addr::unchecked("Contract2"),
            msg: to_binary("").unwrap(),
        },
        Execution {
            order: 2,
            contract: Addr::unchecked("Contract2"),
            msg: to_binary("").unwrap(),
        },
        Execution {
            order: 3,
            contract: Addr::unchecked("Contract2"),
            msg: to_binary("").unwrap(),
        },
    ]);
}
