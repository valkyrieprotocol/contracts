use cosmwasm_std::{Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::proxy::execute_msgs::DexType;
use valkyrie::test_utils::{expect_unauthorized_err};

use crate::executions::{update_config};
use crate::states::{Config};
use valkyrie::test_constants::campaign::{ADMIN2};
use valkyrie::test_constants::proxy::{ADMIN, proxy_env, proxy_sender};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    dex_type: Option<DexType>,
) -> ContractResult<Response> {
    update_config(
        deps.as_mut(),
        env,
        info,
        dex_type,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    dex_type: Option<DexType>,
) -> (Env, MessageInfo, Response) {
    let env = proxy_env();
    let info = proxy_sender(ADMIN);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        dex_type,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(
        &mut deps,
        None
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.fixed_dex, None);

    will_success(
        &mut deps,
        Some(DexType::Astroport)
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.fixed_dex, Some(DexType::Astroport));

    will_success(
        &mut deps,
        None
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.fixed_dex, Some(DexType::Astroport));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        proxy_env(),
        proxy_sender(ADMIN2),
        None,
    );

    expect_unauthorized_err(&result);
}