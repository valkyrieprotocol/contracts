use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use valkyrie::common::ContractResult;
use crate::executions::update_admin;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::campaign_admin_sender;
use crate::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    address: String,
) -> ContractResult<Response> {
    update_admin(deps.as_mut(), env, info, address)
}

pub fn will_success(deps: &mut CustomDeps, address: String) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        address,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let admin = "NewAdmin".to_string();
    will_success(&mut deps, admin.clone());

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admin, Addr::unchecked(admin));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        "NewAdmin".to_string(),
    );

    expect_unauthorized_err(&result);
}
