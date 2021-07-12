use cosmwasm_std::{Env, MessageInfo, Response, Uint128};

use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, default_sender, expect_generic_err, expect_unauthorized_err};

use crate::executions::update_distribution_config;
use crate::states::DistributionConfig;
use crate::tests::{campaign_admin_sender, TOKEN_CONTRACT};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
    amounts: Vec<Uint128>,
) -> ContractResult<Response> {
    update_distribution_config(deps.as_mut(), env, info, denom, amounts)
}

pub fn will_success(
    deps: &mut CustomDeps,
    denom: Denom,
    amounts: Vec<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        denom,
        amounts,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let denom = Denom::Token(TOKEN_CONTRACT.to_string());
    let amounts = vec![Uint128(100), Uint128(50), Uint128(50)];
    will_success(&mut deps, denom.clone(), amounts.clone());

    let config = DistributionConfig::load(&deps.storage).unwrap();
    assert_eq!(config, DistributionConfig {
        denom: denom.to_cw20(&deps.api),
        amounts,
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        Denom::Token(TOKEN_CONTRACT.to_string()),
        vec![],
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_after_activation() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        Denom::Token(TOKEN_CONTRACT.to_string()),
        vec![],
    );

    expect_generic_err(&result, "Only modifiable in pending status");
}

#[test]
fn failed_invalid_amounts() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    will_success(
        &mut deps,
        Denom::Token(TOKEN_CONTRACT.to_string()),
        vec![Uint128::zero(), Uint128::from(100u64)],
    );

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        Denom::Token(TOKEN_CONTRACT.to_string()),
        vec![],
    );
    expect_generic_err(&result, "Invalid reward scheme");

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        Denom::Token(TOKEN_CONTRACT.to_string()),
        vec![Uint128::zero(), Uint128::zero()],
    );
    expect_generic_err(&result, "Invalid reward scheme");
}
