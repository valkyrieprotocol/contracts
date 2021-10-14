use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, Uint128, Binary, to_binary};
use valkyrie::common::ContractResult;
use crate::executions::register_distribution;
use valkyrie::test_utils::{expect_unauthorized_err, expect_overflow_err};
use crate::states::{Distribution, ContractState};
use valkyrie::test_constants::distributor::{distributor_env, MANAGING_TOKEN, DISTRIBUTOR};
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_constants::default_sender;
use valkyrie::governance::execute_msgs::Cw20HookMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    start_height: u64,
    end_height: u64,
    recipient: String,
    amount: Uint128,
    message: Option<Binary>,
) -> ContractResult<Response> {
    register_distribution(
        deps.as_mut(),
        env,
        info,
        start_height,
        end_height,
        recipient,
        amount,
        message,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    start_height: u64,
    end_height: u64,
    recipient: String,
    amount: Uint128,
    message: Option<Binary>,
) -> (Env, MessageInfo, Response) {
    let env = distributor_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        start_height,
        end_height,
        recipient,
        amount,
        message,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(10000)),
    ])]);

    super::instantiate::default(&mut deps);

    will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
        Some(to_binary(&Cw20HookMsg::StakeGovernanceToken {}).unwrap()),
    );

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.distribution_count, 1);
    assert_eq!(state.locked_amount, Uint128::new(10000));

    let distribution = Distribution::may_load(&deps.storage, 1).unwrap().unwrap();
    assert_eq!(distribution, Distribution {
        id: 1,
        start_height: 20000,
        end_height: 30000,
        recipient: Addr::unchecked("Recipient"),
        amount: Uint128::new(10000),
        distributed_amount: Uint128::zero(),
        message: Some(to_binary(&Cw20HookMsg::StakeGovernanceToken {}).unwrap()),
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        distributor_env(),
        default_sender(),
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
        None,
    );
    expect_unauthorized_err(&result);
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(9999)),
    ])]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        distributor_env(),
        governance_sender(),
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
        None,
    );
    expect_overflow_err(&result)
}
