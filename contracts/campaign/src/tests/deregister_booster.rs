use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response};
use valkyrie::common::ContractResult;
use crate::executions::deregister_booster;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::distributor_sender;
use crate::states::BoosterState;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    deregister_booster(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = distributor_sender();

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);
    super::register_booster::default(&mut deps);

    will_success(&mut deps);
    assert!(BoosterState::may_load(&deps.storage).unwrap().is_none());
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);
    super::register_booster::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
    );

    expect_unauthorized_err(&result);
}
