use cosmwasm_std::{Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
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
) -> ContractResult<Response> {
    update_campaign_info(deps.as_mut(), env, info, title, url, description)
}

pub fn will_success(
    deps: &mut CustomDeps,
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
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

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        Some(url.clone()),
    );

    let campaign_info = CampaignInfo::load(&deps.storage).unwrap();
    assert_eq!(campaign_info.title, title);
    assert_eq!(campaign_info.description, description);
    assert_eq!(campaign_info.url, url);
}

#[test]
fn succeed_update_info_after_activation() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let title = "Title2".to_string();
    let description = "Desc2".to_string();

    will_success(
        &mut deps,
        Some(title.clone()),
        Some(description.clone()),
        None,
    );

    let campaign_info = CampaignInfo::load(&deps.storage).unwrap();
    assert_eq!(campaign_info.title, title);
    assert_eq!(campaign_info.description, description);
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
    );

    expect_unauthorized_err(&result);
}
