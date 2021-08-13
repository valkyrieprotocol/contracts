use crate::tests::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Uint128};
use crate::executions::{ExecuteResult, update_requirement};
use cosmwasm_std::testing::{mock_env, mock_info};
use crate::tests::admin_sender;
use crate::states::{QualifierConfig, Requirement};
use crate::errors::ContractError;
use valkyrie_qualifier::QualifiedContinueOption;
use cw20::Denom;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    continue_option_on_fail: Option<QualifiedContinueOption>,
    min_token_balances: Option<Vec<(Denom, Uint128)>>,
    min_luna_staking: Option<Uint128>,
) -> ExecuteResult {
    update_requirement(
        deps.as_mut(),
        env,
        info,
        continue_option_on_fail,
        min_token_balances,
        min_luna_staking,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    continue_option_on_fail: Option<QualifiedContinueOption>,
    min_token_balances: Option<Vec<(Denom, Uint128)>>,
    min_luna_staking: Option<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        continue_option_on_fail,
        min_token_balances,
        min_luna_staking,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let continue_option_on_fail = QualifiedContinueOption::ExecuteOnly;
    let min_token_balances = vec![(Denom::Native("ukrw".to_string()), Uint128::new(500))];
    let min_luna_staking = Uint128::new(300);

    will_success(
        &mut deps,
        Some(continue_option_on_fail.clone()),
        Some(min_token_balances.clone()),
        Some(min_luna_staking.clone()),
    );

    let config = QualifierConfig::load(&deps.storage).unwrap();
    assert_eq!(config.continue_option_on_fail, continue_option_on_fail);

    let requirement = Requirement::load(&deps.storage).unwrap();
    assert_eq!(requirement.min_token_balances, min_token_balances);
    assert_eq!(requirement.min_luna_staking, min_luna_staking);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("AnySender", &[]),
        None,
        None,
        None,
    );
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized {});
}