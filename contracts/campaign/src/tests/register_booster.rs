use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, coin};
use valkyrie::common::ContractResult;
use crate::executions::register_booster;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_already_exists_err};
use crate::tests::{distributor_sender, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE};
use crate::states::BoosterState;

pub const DROP_BOOSTER_AMOUNT: Uint128 = Uint128(1000);
pub const ACTIVITY_BOOSTER_AMOUNT: Uint128 = Uint128(8000);
pub const PLUS_BOOSTER_AMOUNT: Uint128 = Uint128(1000);

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    drop_booster_amount: Uint128,
    activity_booster_amount: Uint128,
    plus_booster_amount: Uint128,
) -> ContractResult<Response> {
    register_booster(
        deps.as_mut(),
        env,
        info,
        drop_booster_amount,
        activity_booster_amount,
        plus_booster_amount,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    drop_booster_amount: Uint128,
    activity_booster_amount: Uint128,
    plus_booster_amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = distributor_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        drop_booster_amount,
        activity_booster_amount,
        plus_booster_amount,
    ).unwrap();

    (env, info, response)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    will_success(
        deps,
        DROP_BOOSTER_AMOUNT,
        ACTIVITY_BOOSTER_AMOUNT,
        PLUS_BOOSTER_AMOUNT,
    )
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Participator", None);

    let (env, _, _) = default(&mut deps);

    let booster_state = BoosterState::load(&deps.storage).unwrap();
    assert_eq!(booster_state, BoosterState {
        drop_booster_amount: DROP_BOOSTER_AMOUNT,
        drop_booster_left_amount: DROP_BOOSTER_AMOUNT,
        drop_booster_participations: 1,
        activity_booster_amount: ACTIVITY_BOOSTER_AMOUNT,
        activity_booster_left_amount: ACTIVITY_BOOSTER_AMOUNT,
        plus_booster_amount: PLUS_BOOSTER_AMOUNT,
        plus_booster_left_amount: PLUS_BOOSTER_AMOUNT,
        boosted_at: env.block.time,
    });
}

#[test]
fn succeed_after_finished_boosting() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    default(&mut deps);
    assert!(BoosterState::may_load(&deps.storage).unwrap().is_some());

    super::deregister_booster::will_success(&mut deps);

    default(&mut deps);
    assert!(BoosterState::may_load(&deps.storage).unwrap().is_some());
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        DROP_BOOSTER_AMOUNT,
        ACTIVITY_BOOSTER_AMOUNT,
        PLUS_BOOSTER_AMOUNT,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_already_boosting() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    default(&mut deps);
    assert!(BoosterState::may_load(&deps.storage).unwrap().is_some());

    let result = exec(
        &mut deps,
        contract_env(),
        distributor_sender(),
        DROP_BOOSTER_AMOUNT,
        ACTIVITY_BOOSTER_AMOUNT,
        PLUS_BOOSTER_AMOUNT,
    );

    expect_already_exists_err(&result);
}
