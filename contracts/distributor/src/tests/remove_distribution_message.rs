use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Uint128, to_binary};
use valkyrie::common::ContractResult;
use crate::executions::remove_distribution_message;
use valkyrie::test_utils::expect_unauthorized_err;
use crate::states::Distribution;
use valkyrie::test_constants::distributor::{distributor_env, MANAGING_TOKEN, DISTRIBUTOR, RECIPIENT};
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_constants::default_sender;
use valkyrie::lp_staking::execute_msgs::Cw20HookMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> ContractResult<Response> {
    remove_distribution_message(
        deps.as_mut(),
        env,
        info,
        id,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    id: u64,
) -> (Env, MessageInfo, Response) {
    let env = distributor_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        id,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(20000)),
    ])]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        RECIPIENT.to_string(),
        Uint128::new(10000),
        Some(to_binary(&Cw20HookMsg::Bond {}).unwrap()),
    );

    will_success(&mut deps, 1);

    let distribution = Distribution::may_load(&deps.storage, 1).unwrap().unwrap();
    assert_eq!(distribution.message, None);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(10000)),
    ])]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        RECIPIENT.to_string(),
        Uint128::new(10000),
        None,
    );

    let result = exec(
        &mut deps,
        distributor_env(),
        default_sender(),
        1,
    );
    expect_unauthorized_err(&result);
}
