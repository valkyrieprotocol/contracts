use cosmwasm_std::{Decimal, Env, MessageInfo, Response, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::{campaign_admin_sender, campaign_env};
use valkyrie::test_constants::default_sender;
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::executions::update_reward_config;
use crate::states::RewardConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    participation_reward_amount: Option<Uint128>,
    participation_reward_distribution_schedule: Option<Vec<(u64, u64, Decimal)>>,
    referral_reward_amounts: Option<Vec<Uint128>>,
    referral_reward_lock_period: Option<u64>,
) -> ContractResult<Response> {
    update_reward_config(
        deps.as_mut(),
        env,
        info,
        participation_reward_amount,
        participation_reward_distribution_schedule,
        referral_reward_amounts,
        referral_reward_lock_period,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    participation_reward_amount: Option<Uint128>,
    participation_reward_distribution_schedule: Option<Vec<(u64, u64, Decimal)>>,
    referral_reward_amounts: Option<Vec<Uint128>>,
    referral_reward_lock_period: Option<u64>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        participation_reward_amount,
        participation_reward_distribution_schedule,
        referral_reward_amounts,
        referral_reward_lock_period,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let participation_reward_amount = Uint128::new(122);
    let participation_reward_distribution_schedule = vec![
        (1000, 1000, Decimal::percent(50)),
        (1000, 2000, Decimal::percent(30)),
        (2000, 3000, Decimal::percent(20)),
    ];
    let referral_reward_amounts = vec![
        Uint128::new(100),
        Uint128::new(50),
        Uint128::new(50),
    ];
    let referral_reward_lock_period = 99u64;

    will_success(
        &mut deps,
        Some(participation_reward_amount.clone()),
        Some(participation_reward_distribution_schedule.clone()),
        Some(referral_reward_amounts.clone()),
        Some(referral_reward_lock_period.clone()),
    );

    let config = RewardConfig::load(&deps.storage).unwrap();
    assert_eq!(config.participation_reward_amount, participation_reward_amount);
    assert_eq!(config.participation_reward_lock_period, 0);
    assert_eq!(config.participation_reward_distribution_schedule, participation_reward_distribution_schedule);
    assert_eq!(config.referral_reward_amounts, referral_reward_amounts);
    assert_eq!(config.referral_reward_lock_period, referral_reward_lock_period);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
        None,
        None,
        None,
        None,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_after_activation() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    super::update_activation::will_success(&mut deps, true);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        None,
        None,
    );

    expect_generic_err(&result, "Only modifiable in pending status");
}

#[test]
fn failed_invalid_amounts() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(
        &mut deps,
        None,
        None,
        Some(vec![Uint128::zero(), Uint128::from(100u64)]),
        None,
    );

    will_success(
        &mut deps,
        Some(Uint128::zero()),
        None,
        None,
        None,
    );

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        Some(vec![]),
        None,
    );
    expect_generic_err(&result, "Invalid reward scheme");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        None,
        Some(vec![Uint128::zero(), Uint128::zero()]),
        None,
    );
    expect_generic_err(&result, "Invalid reward scheme");
}
