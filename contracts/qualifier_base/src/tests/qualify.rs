use cosmwasm_std::{coin, Env, from_binary, MessageInfo, Response};
use cosmwasm_std::testing::{mock_env, mock_info};

use valkyrie_qualifier::{QualificationMsg, QualificationResult, QualifiedContinueOption};

use crate::executions::{ExecuteResult, qualify};
use crate::tests::{CONTINUE_OPTION_ON_FAIL, MIN_LUNA_STAKING, MIN_TOKEN_BALANCE_AMOUNT};
use crate::tests::mock_querier::{custom_deps, CustomDeps};
use valkyrie::campaign::query_msgs::ActorResponse;

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


    let (_, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, QualifiedContinueOption::Eligible);

    deps.querier.with_balance(&[("Actor2", &[
        (coin(MIN_TOKEN_BALANCE_AMOUNT.u128(), "uluna")),
    ])]);

    deps.querier.plus_delegation("Actor2", "Delegator", MIN_LUNA_STAKING);

    let mut actor = ActorResponse::new("Actor2".to_string(), None);
    actor.participation_count = 1;
    deps.querier.with_actor(actor);


    let (_, _, response) = will_success(&mut deps, "Actor2".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, QualifiedContinueOption::Eligible);
}

#[test]
fn dissatisfy_min_token_balances() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_delegation("Actor", "Delegator", MIN_LUNA_STAKING);

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

    let (_, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, CONTINUE_OPTION_ON_FAIL);
}

#[test]
fn dissatisfy_participation_limit() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.with_balance(&[("Actor", &[
        (coin(MIN_TOKEN_BALANCE_AMOUNT.u128(), "uluna")),
    ])]);

    deps.querier.plus_delegation("Actor", "Delegator", MIN_LUNA_STAKING);

    let mut actor = ActorResponse::new("Actor".to_string(), None);
    actor.participation_count = 2;
    deps.querier.with_actor(actor);

    let (_, _, response) = will_success(&mut deps, "Actor".to_string());
    let result: QualificationResult = from_binary(&response.data.unwrap()).unwrap();
    assert_eq!(result.continue_option, CONTINUE_OPTION_ON_FAIL);
}
