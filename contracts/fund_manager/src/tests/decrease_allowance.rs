use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, Addr};
use valkyrie::common::ContractResult;
use crate::executions::decrease_allowance;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_generic_err};
use crate::tests::{campaign_manager_sender, TOKEN_CONTRACT};
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
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
    let env = contract_env();
    let info = campaign_manager_sender();

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
    let mut deps = custom_deps(&[]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128::new(10000))],
    )]);

    let address = Addr::unchecked("Address");

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
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        "Address".to_string(),
        None,
    );
    expect_unauthorized_err(&result);
}

#[test]
fn failed_insufficient_remain_amount() {
    let mut deps = custom_deps(&[]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128::new(10000))],
    )]);

    let address = Addr::unchecked("Address");

    super::instantiate::default(&mut deps);
    super::increase_allowance::will_success(&mut deps, address.to_string(), Uint128::new(1000));

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_manager_sender(),
        address.to_string(),
        Some(Uint128::new(1001)),
    );

    expect_generic_err(&result, "Insufficient remain amount");
}