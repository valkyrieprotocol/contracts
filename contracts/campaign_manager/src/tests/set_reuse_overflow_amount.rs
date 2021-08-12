use cosmwasm_std::{Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::campaign_manager_env;
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_utils::expect_unauthorized_err;

use crate::executions::set_reuse_overflow_amount;
use crate::states::ReferralRewardLimitOption;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    set_reuse_overflow_amount(
        deps.as_mut(),
        env,
        info,
    )
}

pub fn will_success(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = governance_sender();

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(&mut deps);

    let option = ReferralRewardLimitOption::load(&deps.storage).unwrap();
    assert_eq!(option.overflow_amount_recipient, None);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
    );
    expect_unauthorized_err(&result);
}
