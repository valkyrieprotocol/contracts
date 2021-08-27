use crate::tests::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use crate::executions::{ExecuteResult, update_config};
use cosmwasm_std::testing::{mock_env, mock_info};
use crate::tests::admin_sender;
use crate::states::QualifierConfig;
use crate::errors::ContractError;
use valkyrie_qualifier::QualifiedContinueOption;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    continue_option_on_fail: Option<QualifiedContinueOption>,
) -> ExecuteResult {
    update_config(
        deps.as_mut(),
        env,
        info,
        admin,
        continue_option_on_fail,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    admin: Option<String>,
    continue_option_on_fail: Option<QualifiedContinueOption>,
) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        admin,
        continue_option_on_fail,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let continue_option_on_fail = QualifiedContinueOption::ExecuteOnly;

    will_success(
        &mut deps,
        Some("NewAdmin".to_string()),
        Some(continue_option_on_fail.clone()),
    );

    let config = QualifierConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked("NewAdmin"));
    assert_eq!(config.continue_option_on_fail, continue_option_on_fail);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("AnySender", &[]),
        None,
        None,
    );
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized {});
}