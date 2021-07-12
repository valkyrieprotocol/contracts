use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, coin};
use valkyrie::common::ContractResult;
use crate::executions::disable_booster;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::{campaign_manager_sender, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE};
use crate::states::Booster;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    disable_booster(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = campaign_manager_sender();

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Participator1", None);
    super::enable_booster::default(&mut deps);

    will_success(&mut deps);
    assert!(!Booster::is_boosting(&deps.storage).unwrap());
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Participator1", None);
    super::enable_booster::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
    );

    expect_unauthorized_err(&result);
}
