use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response};
use valkyrie::common::{Denom, ContractResult};
use crate::executions::remove_distribution_denom;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_not_found_err};
use crate::tests::governance_sender;
use crate::states::CampaignConfig;
use valkyrie::utils::find;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
) -> ContractResult<Response> {
    remove_distribution_denom(deps.as_mut(), env, info, denom)
}

pub fn will_success(deps: &mut CustomDeps, denom: Denom) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        denom,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let denom = Denom::Token("NewToken".to_string());

    super::add_distribution_denom::will_success(&mut deps, denom.clone());

    will_success(&mut deps, denom.clone());

    let config = CampaignConfig::load(&deps.storage).unwrap();
    assert!(find(&config.distribution_denom_whitelist, |d| denom.to_cw20(&deps.api) == *d).is_none())
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        Denom::Token("NewToken".to_string()),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_not_found() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    super::add_distribution_denom::will_success(&mut deps, Denom::Token("NewToken".to_string()));

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        Denom::Token("NewToken2".to_string()),
    );
    expect_not_found_err(&result);
}
