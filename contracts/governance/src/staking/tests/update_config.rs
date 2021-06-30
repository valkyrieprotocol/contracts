use cosmwasm_std::{Env, MessageInfo, Response};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_info};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use crate::staking::executions::update_config;
use crate::staking::states::StakingConfig;
use crate::tests::{default_env, default_info, expect_unauthorized_err, init_default, WITHDRAW_DELAY};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, withdraw_delay: Option<u64>) -> ContractResult<Response> {
    update_config(deps.as_mut(), env, info, withdraw_delay)
}

pub fn will_success(deps: &mut CustomDeps, withdraw_delay: Option<u64>) -> (Env, MessageInfo, Response) {
    let env = default_env();
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        withdraw_delay,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let withdraw_delay = WITHDRAW_DELAY + 100;

    will_success(&mut deps, Some(withdraw_delay));

    let config = StakingConfig::load(&deps.storage).unwrap();
    assert_eq!(config.withdraw_delay, withdraw_delay);
    assert_ne!(config.withdraw_delay, WITHDRAW_DELAY);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let env = default_env();
    let info = default_info();

    let result = exec(&mut deps, env, info, None);

    expect_unauthorized_err(&result);
}