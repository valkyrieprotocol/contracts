use cosmwasm_std::{Addr, Env, MessageInfo, Response, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::community::{COMMUNITY, community_env, MANAGING_TOKEN};
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::executions::decrease_allowance;
use crate::states::{Allowance, ContractState};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    address: String,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    decrease_allowance(
        deps.as_mut(),
        env,
        info,
        address,
        amount,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    address: String,
    amount: Option<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = community_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        address,
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(10000))],
    )]);

    let address = Addr::unchecked("terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n");

    super::instantiate::default(&mut deps);
    super::increase_allowance::will_success(&mut deps, address.to_string(), Uint128::new(1000));

    will_success(&mut deps, address.to_string(), Some(Uint128::new(100)));

    let allowance = Allowance::load(&deps.storage, &address).unwrap();
    assert_eq!(allowance, Allowance {
        address: address.clone(),
        allowed_amount: Uint128::new(900),
        remain_amount: Uint128::new(900),
    });

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.remain_allowance_amount, Uint128::new(900));

    will_success(&mut deps, address.to_string(), None);

    let allowance = Allowance::may_load(&deps.storage, &address).unwrap();
    assert!(allowance.is_none());

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.remain_allowance_amount, Uint128::zero());
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        community_env(),
        default_sender(),
        "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n".to_string(),
        None,
    );
    expect_unauthorized_err(&result);
}

#[test]
fn failed_insufficient_remain_amount() {
    let mut deps = custom_deps();
    deps.querier.with_token_balances(&[(
        MANAGING_TOKEN,
        &[(COMMUNITY, &Uint128::new(10000))],
    )]);

    let address = Addr::unchecked("terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n");

    super::instantiate::default(&mut deps);
    super::increase_allowance::will_success(&mut deps, address.to_string(), Uint128::new(1000));

    let result = exec(
        &mut deps,
        community_env(),
        governance_sender(),
        address.to_string(),
        Some(Uint128::new(1001)),
    );

    expect_generic_err(&result, "Insufficient remain amount");
}