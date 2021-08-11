use crate::tests::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, coin, from_binary, Addr};
use crate::executions::{ExecuteResult, qualify};
use valkyrie_qualifier::{QualificationMsg, QualificationResult, QualifiedContinueOption};
use cosmwasm_std::testing::{mock_env, mock_info};
use crate::tests::{MIN_LUNA_STAKING, COLLATERAL_AMOUNT, CONTINUE_OPTION_ON_FAIL, COLLATERAL_LOCK_PERIOD, MIN_TOKEN_BALANCE_AMOUNT};
use crate::states::Collateral;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    actor: String,
) -> ExecuteResult {
    let msg = QualificationMsg {
        campaign: "Campaign".to_string(),
        sender: "Sender".to_string(),
        actor,
        referrer: None,
    };

    qualify(deps.as_mut(), env, info, msg)
}

pub fn will_success(deps: &mut CustomDeps, actor: String) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info("Campaign", &[]);

    let response = exec(deps, env.clone(), info.clone(), actor).unwrap();

    (env, info, response)
}

#[test]
fn satisfy() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.with_balance(&[("Actor", &[
        (coin(MIN_TOKEN_BALANCE_AMOUNT.u128(), "uluna")),
    ])]);

    deps.querier.plus_delegation("Actor", "Delegator", MIN_LUNA_STAKING);

    super::deposit_collateral::will_success(&mut deps, "Actor", COLLATERAL_AMOUNT);

    let (env, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, QualifiedContinueOption::Eligible);

    let collateral = Collateral::load(&deps.storage, &Addr::unchecked("Actor")).unwrap();
    assert_eq!(collateral.locked_amounts, vec![(COLLATERAL_AMOUNT, COLLATERAL_LOCK_PERIOD + env.block.height)]);
}

#[test]
fn dissatisfy_min_token_balances() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_delegation("Actor", "Delegator", MIN_LUNA_STAKING);

    super::deposit_collateral::will_success(&mut deps, "Actor", COLLATERAL_AMOUNT);

    let (_, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, CONTINUE_OPTION_ON_FAIL);
}

#[test]
fn dissatisfy_min_luna_staking() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.with_balance(&[("Actor", &[
        (coin(MIN_TOKEN_BALANCE_AMOUNT.u128(), "uluna")),
    ])]);

    super::deposit_collateral::will_success(&mut deps, "Actor", COLLATERAL_AMOUNT);

    let (_, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, CONTINUE_OPTION_ON_FAIL);
}

#[test]
fn dissatisfy_collateral() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.with_balance(&[("Actor", &[
        (coin(MIN_TOKEN_BALANCE_AMOUNT.u128(), "uluna")),
    ])]);

    deps.querier.plus_delegation("Actor", "Delegator", MIN_LUNA_STAKING);

    let (_, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, CONTINUE_OPTION_ON_FAIL);
}
