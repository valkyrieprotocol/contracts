use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use valkyrie::common::ContractResult;
use crate::executions::update_config;
use valkyrie::test_utils::expect_unauthorized_err;
use crate::states::ContractConfig;
use valkyrie::test_constants::distributor::distributor_env;
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_constants::default_sender;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
) -> ContractResult<Response> {
    update_config(
        deps.as_mut(),
        env,
        info,
        admin,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    admin: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = distributor_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        admin,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let admin = "Admin1".to_string();

    will_success(
        &mut deps,
        Some(admin.clone()),
    );

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked(admin));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        distributor_env(),
        default_sender(),
        None,
    );
    expect_unauthorized_err(&result);
}
