use cosmwasm_std::{Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::proxy::execute_msgs::DexType;
use valkyrie::test_constants::proxy::{ADMIN, ADMIN2, proxy_env, proxy_sender};
use valkyrie::test_utils::{expect_unauthorized_err};

use crate::executions::{clear_fixed_dex};
use crate::states::{Config};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    clear_fixed_dex(
        deps.as_mut(),
        env,
        info,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
) -> (Env, MessageInfo, Response) {
    let env = proxy_env();
    let info = proxy_sender(ADMIN);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_config::will_success(&mut deps, Some(DexType::Astroport));

    will_success(
        &mut deps,
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.fixed_dex, None);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        proxy_env(),
        proxy_sender(ADMIN2),
    );

    expect_unauthorized_err(&result);
}