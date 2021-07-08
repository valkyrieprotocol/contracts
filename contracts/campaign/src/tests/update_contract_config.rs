use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use valkyrie::common::ContractResult;
use crate::executions::update_contract_config;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::campaign_admin_sender;
use crate::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    proxies: Option<Vec<String>>,
) -> ContractResult<Response> {
    update_contract_config(deps.as_mut(), env, info, admin, proxies)
}

pub fn will_success(
    deps: &mut CustomDeps,
    admin: Option<String>,
    proxies: Option<Vec<String>>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        admin,
        proxies,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let admin = "NewAdmin".to_string();
    let proxy = "Proxy".to_string();
    will_success(&mut deps, Some(admin.clone()), Some(vec![proxy.clone()]));

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked(admin));
    assert_eq!(config.proxies, vec![Addr::unchecked(proxy)]);
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
