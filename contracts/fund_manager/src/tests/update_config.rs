use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use valkyrie::common::ContractResult;
use crate::executions::update_config;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::governance_sender;
use crate::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    terraswap_router: Option<String>,
) -> ContractResult<Response> {
    update_config(deps.as_mut(), env, info, terraswap_router)
}

pub fn will_success(
    deps: &mut CustomDeps,
    terraswap_router: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        terraswap_router,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let terraswap_router = "TerraSwapRouterChanged";

    will_success(&mut deps, Some(terraswap_router.to_string()));

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config.terraswap_router, Addr::unchecked(terraswap_router));
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
    );
    expect_unauthorized_err(&result);
}
