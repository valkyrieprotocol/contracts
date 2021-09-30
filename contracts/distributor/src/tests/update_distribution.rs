use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, Uint128};
use valkyrie::common::ContractResult;
use crate::executions::update_distribution;
use valkyrie::test_utils::{expect_unauthorized_err, expect_overflow_err, set_height, expect_generic_err};
use crate::states::{Distribution, ContractState};
use valkyrie::test_constants::distributor::{distributor_env, MANAGING_TOKEN, DISTRIBUTOR};
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_constants::default_sender;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    id: u64,
    start_height: Option<u64>,
    end_height: Option<u64>,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    update_distribution(
        deps.as_mut(),
        env,
        info,
        id,
        start_height,
        end_height,
        amount,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    id: u64,
    start_height: Option<u64>,
    end_height: Option<u64>,
    amount: Option<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = distributor_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        id,
        start_height,
        end_height,
        amount,
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
        "Recipient".to_string(),
        Uint128::new(10000),
    );

    will_success(
        &mut deps,
        1,
        Some(20001),
        Some(31000),
        Some(Uint128::new(20000)),
    );

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.distribution_count, 1);
    assert_eq!(state.locked_amount, Uint128::new(20000));

    let distribution = Distribution::may_load(&deps.storage, 1).unwrap().unwrap();
    assert_eq!(distribution, Distribution {
        id: 1,
        start_height: 20001,
        end_height: 31000,
        recipient: Addr::unchecked("Recipient"),
        amount: Uint128::new(20000),
        distributed_amount: Uint128::zero(),
    });
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
        "Recipient".to_string(),
        Uint128::new(10000),
    );

    let result = exec(
        &mut deps,
        distributor_env(),
        default_sender(),
        1,
        None,
        None,
        None,
    );
    expect_unauthorized_err(&result);
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(15000)),
    ])]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
    );

    let result = exec(
        &mut deps,
        distributor_env(),
        governance_sender(),
        1,
        None,
        None,
        Some(Uint128::new(15001)),
    );
    expect_overflow_err(&result)
}

#[test]
fn failed_less_than_released_amount() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(10000)),
    ])]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
    );

    let mut env = distributor_env();
    set_height(&mut env, 25000);

    let result = exec(
        &mut deps,
        env,
        governance_sender(),
        1,
        None,
        None,
        Some(Uint128::new(4999)),
    );
    expect_generic_err(&result, "amount must be less than released_amount");
}
