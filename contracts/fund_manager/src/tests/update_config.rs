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
    admins: Option<Vec<String>>,
    terraswap_router: Option<String>,
) -> ContractResult<Response> {
    update_config(deps.as_mut(), env, info, admins, terraswap_router)
}

pub fn will_success(
    deps: &mut CustomDeps,
    admins: Option<Vec<String>>,
    terraswap_router: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        admins,
        terraswap_router,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let admins = vec!["Admin1".to_string(), "Admins2".to_string()];
    let terraswap_router = "TerraSwapRouterChanged";

    will_success(&mut deps, Some(admins.clone()), Some(terraswap_router.to_string()));

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admins, admins.iter().map(|v| Addr::unchecked(v)).collect::<Vec<Addr>>());
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
        None,
    );
    expect_unauthorized_err(&result);
}
