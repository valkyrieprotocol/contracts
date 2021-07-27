use cosmwasm_std::{Addr, Env, MessageInfo, Response, Uint128};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_info};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use crate::staking::executions::stake_governance_token;
use crate::staking::states::{StakerState, StakingState};
use crate::tests::{init_default, GOVERNANCE_TOKEN};
use valkyrie::test_utils::{contract_env, expect_generic_err, expect_unauthorized_err};

pub const STAKER1: &str = "Staker1";
pub const STAKER1_STAKE_AMOUNT: Uint128 = Uint128::new(10u128);

pub const STAKER2: &str = "Staker2";
pub const STAKER2_STAKE_AMOUNT: Uint128 = Uint128::new(10u128);

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, sender: Addr, amount: Uint128) -> ContractResult<Response> {
    deps.querier.plus_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(MOCK_CONTRACT_ADDR, &amount)],
    )]);

    stake_governance_token(deps.as_mut(), env, info, sender, amount)
}

pub fn will_success(deps: &mut CustomDeps, staker: &str, amount: Uint128) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(GOVERNANCE_TOKEN, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        Addr::unchecked(staker),
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    will_success(&mut deps, STAKER1, STAKER1_STAKE_AMOUNT);

    let staking_state = StakingState::load(&deps.storage).unwrap();
    assert_eq!(staking_state.total_share, STAKER1_STAKE_AMOUNT);

    let staker_state = StakerState::load(&deps.storage, &Addr::unchecked(STAKER1)).unwrap();
    assert_eq!(staker_state.share, STAKER1_STAKE_AMOUNT);
}

#[test]
fn failed_insufficient_funds() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(GOVERNANCE_TOKEN, &[]),
        Addr::unchecked(STAKER1),
        Uint128::zero(),
    );

    expect_generic_err(&result, "Insufficient funds sent");
}

#[test]
fn failed_wrong_token() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info("Another Token", &[]),
        Addr::unchecked(STAKER1),
        STAKER1_STAKE_AMOUNT,
    );

    expect_unauthorized_err(&result);
}
