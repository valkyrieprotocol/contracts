use cosmwasm_std::{Env, MessageInfo, Response, to_binary, Addr};

use valkyrie::common::{ContractResult, ExecutionMsg, Execution};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, default_sender, expect_generic_err, expect_unauthorized_err};

use crate::executions::update_campaign_info;
use crate::states::CampaignInfo;
use crate::tests::campaign_admin_sender;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    executions: Option<Vec<ExecutionMsg>>,
) -> ContractResult<Response> {
    update_campaign_info(
        deps.as_mut(),
        env,
        info,
        title,
        description,
        url,
        parameter_key,
        executions,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    parameter_key: Option<String>,
    executions: Option<Vec<ExecutionMsg>>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        title,
        description,
        url,
        parameter_key,
        executions,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let title = "Title2".to_string();
    let description = "Desc2".to_string();
    let url = "https://url2.url".to_string();
    let parameter_key = "vkr2".to_string();
    let executions = vec![
        ExecutionMsg {
            order: 1,
            contract: "ExecContract".to_string(),
            msg: to_binary("").unwrap(),
        },
    ];

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        Some(url.clone()),
        Some(parameter_key.clone()),
        Some(executions.clone()),
    );

    let campaign_info = CampaignInfo::load(&deps.storage).unwrap();
    assert_eq!(campaign_info.title, title);
    assert_eq!(campaign_info.description, description);
    assert_eq!(campaign_info.url, url);
    assert_eq!(campaign_info.parameter_key, parameter_key);
    assert_eq!(campaign_info.executions, executions.iter().map(|e| Execution {
        order: e.order,
        contract: Addr::unchecked(e.contract.as_str()),
        msg: e.msg.clone(),
    }).collect::<Vec<Execution>>());
}

#[test]
fn succeed_update_info_after_activation() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let title = "Title2".to_string();
    let description = "Desc2".to_string();
    let executions = vec![
        ExecutionMsg {
            order: 1,
            contract: "ExecContract".to_string(),
            msg: to_binary("").unwrap(),
        },
    ];

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        None,
        None,
        Some(executions.clone()),
    );

    let campaign_info = CampaignInfo::load(&deps.storage).unwrap();
    assert_eq!(campaign_info.title, title);
    assert_eq!(campaign_info.description, description);
    assert_eq!(campaign_info.executions, executions.iter().map(|e| Execution {
        order: e.order,
        contract: Addr::unchecked(e.contract.as_str()),
        msg: e.msg.clone(),
    }).collect::<Vec<Execution>>());
}

#[test]
fn failed_update_url_after_activation() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        None,
        None,
        Some("https://url2.url".to_string()),
        None,
        None,
    );

    expect_generic_err(&result, "Only modifiable in pending status");

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        None,
        None,
        None,
        Some("vkr2".to_string()),
        None,
    );

    expect_generic_err(&result, "Only modifiable in pending status");
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        None,
        None,
        None,
        None,
        None,
    );

    expect_unauthorized_err(&result);
}
