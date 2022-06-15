use cosmwasm_std::{Addr, Env, MessageInfo, Response};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::*;
use valkyrie::test_utils::expect_unauthorized_err;
use crate::staking::executions::update_staking_config;
use crate::staking::states::StakingConfig;

use crate::tests::init_default;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    distributor: Option<String>,
) -> ContractResult<Response> {
    update_staking_config(
        deps.as_mut(),
        env,
        info,
        distributor,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    distributor: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = mock_info(GOVERNANCE, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        distributor,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let distributor = "terra17q4lzg70un58uefr2fwu7uxtgvftspr7d0a6p3";

    will_success(&mut deps, Some(distributor.to_string()));

    let config = StakingConfig::load(&deps.storage).unwrap();
    assert_eq!(config, StakingConfig {
        distributor: Some(Addr::unchecked(distributor)),
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        None,
    );

    expect_unauthorized_err(&result);
}