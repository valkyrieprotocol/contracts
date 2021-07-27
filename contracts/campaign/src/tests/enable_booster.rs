use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, coin, Decimal};
use valkyrie::common::ContractResult;
use crate::executions::enable_booster;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_already_exists_err};
use crate::tests::{CAMPAIGN_DISTRIBUTION_DENOM_NATIVE, CAMPAIGN_DISTRIBUTION_AMOUNTS, campaign_manager_sender};
use crate::states::{BoosterState, Booster, DropBooster, ActivityBooster, PlusBooster};
use valkyrie::utils::{to_ratio_uint128, split_uint128};

pub const DROP_BOOSTER_AMOUNT: Uint128 = Uint128::new(1000);
pub const ACTIVITY_BOOSTER_AMOUNT: Uint128 = Uint128::new(8000);
pub const PLUS_BOOSTER_AMOUNT: Uint128 = Uint128::new(1000);
pub const ACTIVITY_BOOSTER_MULTIPLIER_PERCENT: u64 = 80u64;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    drop_booster_amount: Uint128,
    activity_booster_amount: Uint128,
    plus_booster_amount: Uint128,
    activity_booster_multiplier: Decimal,
) -> ContractResult<Response> {
    enable_booster(
        deps.as_mut(),
        env,
        info,
        drop_booster_amount,
        activity_booster_amount,
        plus_booster_amount,
        activity_booster_multiplier,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    drop_booster_amount: Uint128,
    activity_booster_amount: Uint128,
    plus_booster_amount: Uint128,
    activity_booster_multiplier: Decimal,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = campaign_manager_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        drop_booster_amount,
        activity_booster_amount,
        plus_booster_amount,
        activity_booster_multiplier,
    ).unwrap();

    (env, info, response)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    will_success(
        deps,
        DROP_BOOSTER_AMOUNT,
        ACTIVITY_BOOSTER_AMOUNT,
        PLUS_BOOSTER_AMOUNT,
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
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
        recent_booster_id: 1,
    });

    let booster = Booster::load_active(&deps.storage).unwrap();
    assert_eq!(booster, Booster {
        id: 1,
        drop_booster: DropBooster {
            assigned_amount: DROP_BOOSTER_AMOUNT,
            calculated_amount: DROP_BOOSTER_AMOUNT * to_ratio_uint128(&CAMPAIGN_DISTRIBUTION_AMOUNTS.to_vec())[0],
            spent_amount: Uint128::zero(),
            reward_amount: DROP_BOOSTER_AMOUNT,
            reward_amounts: split_uint128(DROP_BOOSTER_AMOUNT, &CAMPAIGN_DISTRIBUTION_AMOUNTS.to_vec()),
            snapped_participation_count: 1,
            snapped_distance_counts: vec![1],
        },
        activity_booster: ActivityBooster {
            assigned_amount: ACTIVITY_BOOSTER_AMOUNT,
            distributed_amount: Uint128::zero(),
            reward_amount: DROP_BOOSTER_AMOUNT * Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
            reward_amounts: split_uint128(
                DROP_BOOSTER_AMOUNT * Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
                &CAMPAIGN_DISTRIBUTION_AMOUNTS.to_vec(),
            ),
        },
        plus_booster: PlusBooster {
            assigned_amount: PLUS_BOOSTER_AMOUNT,
            distributed_amount: Uint128::zero(),
        },
        boosted_at: env.block.time,
        finished_at: None,
    });
}

#[test]
fn succeed_after_finished_boosting() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Participator1", None);

    default(&mut deps);
    assert!(Booster::is_boosting(&deps.storage).unwrap());

    let booster_state = BoosterState::load(&deps.storage).unwrap();
    assert_eq!(booster_state, BoosterState {
        recent_booster_id: 1,
    });

    super::disable_booster::will_success(&mut deps);
    assert!(!Booster::is_boosting(&deps.storage).unwrap());

    let booster_state = BoosterState::load(&deps.storage).unwrap();
    assert_eq!(booster_state, BoosterState {
        recent_booster_id: 1,
    });

    default(&mut deps);
    assert!(Booster::is_boosting(&deps.storage).unwrap());

    let booster_state = BoosterState::load(&deps.storage).unwrap();
    assert_eq!(booster_state, BoosterState {
        recent_booster_id: 2,
    });
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
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_already_boosting() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Participator1", None);

    default(&mut deps);
    assert!(Booster::is_boosting(&deps.storage).unwrap());

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_manager_sender(),
        DROP_BOOSTER_AMOUNT,
        ACTIVITY_BOOSTER_AMOUNT,
        PLUS_BOOSTER_AMOUNT,
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
    );

    expect_already_exists_err(&result);
}

#[test]
fn test_correct_drop_booster_reward() {
    fn test(
        assigned_amount: Uint128,
        distribution_amount: Vec<Uint128>,
        distance_counts: Vec<u64>,
        expect_calculated_amount: Uint128,
        expect_reward_amounts: Vec<Uint128>,
    ) {
        assert_eq!(
            DropBooster::new(
                assigned_amount.clone(),
                distribution_amount.clone(),
                distance_counts[0],
                distance_counts.clone(),
            ),
            DropBooster {
                assigned_amount,
                calculated_amount: expect_calculated_amount,
                spent_amount: Uint128::zero(),
                reward_amount: expect_reward_amounts.iter().sum(),
                reward_amounts: expect_reward_amounts,
                snapped_participation_count: distance_counts[0],
                snapped_distance_counts: distance_counts,
            },
        );
    }

    test(
        Uint128::from(1000u64),
        vec![Uint128::from(5u64), Uint128::from(3u64), Uint128::from(2u64)],
        vec![10, 5, 1],
        Uint128::from(670u64),
        vec![Uint128::from(50u64), Uint128::from(30u64), Uint128::from(20u64)],
    );
}

#[test]
fn test_correct_activity_booster_reward() {
    fn test(
        assigned_amount: Uint128,
        distribution_amounts: Vec<Uint128>,
        drop_booster_reward_amount: Uint128,
        multiplier: Decimal,
        expect_reward_amounts: Vec<Uint128>,
    ) {
        assert_eq!(
            ActivityBooster::new(
                assigned_amount.clone(),
                distribution_amounts,
                drop_booster_reward_amount,
                multiplier,
            ),
            ActivityBooster {
                assigned_amount,
                distributed_amount: Uint128::zero(),
                reward_amount: expect_reward_amounts.iter().sum(),
                reward_amounts: expect_reward_amounts,
            },
        )
    }

    test(
        Uint128::from(8000u64),
        vec![Uint128::from(5u64), Uint128::from(3u64), Uint128::from(2u64)],
        Uint128::from(100u64),
        Decimal::percent(80u64),
        vec![Uint128::from(40u64), Uint128::from(24u64), Uint128::from(16u64)],
    )
}
