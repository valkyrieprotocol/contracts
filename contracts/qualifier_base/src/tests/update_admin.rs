use crate::tests::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use crate::executions::{ExecuteResult, update_admin};
use cosmwasm_std::testing::{mock_env, mock_info};
use crate::tests::admin_sender;
use crate::states::QualifierConfig;
use crate::errors::ContractError;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    new_admin: String,
) -> ExecuteResult {
    update_admin(deps.as_mut(), env, info, new_admin)
}

pub fn will_success(deps: &mut CustomDeps, new_admin: String) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = admin_sender();

    let response = exec(deps, env.clone(), info.clone(), new_admin).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(&mut deps, "NewAdmin".to_string());

    let config = QualifierConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked("NewAdmin"));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("AnySender", &[]),
        "NewAdmin".to_string(),
    );
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized {});
}