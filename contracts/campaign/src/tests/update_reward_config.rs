use cosmwasm_std::{Env, MessageInfo, Response, Uint128};

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
    referral_reward_amounts: Option<Vec<Uint128>>,
) -> ContractResult<Response> {
    update_reward_config(
        deps.as_mut(),
        env,
        info,
        participation_reward_amount,
        referral_reward_amounts,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    participation_reward_amount: Option<Uint128>,
    referral_reward_amounts: Option<Vec<Uint128>>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        participation_reward_amount,
        referral_reward_amounts,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let participation_reward_amount = Uint128::new(122);
    let referral_reward_amounts = vec![
        Uint128::new(100),
        Uint128::new(50),
        Uint128::new(50),
    ];
    will_success(
        &mut deps,
        Some(participation_reward_amount.clone()),
        Some(referral_reward_amounts.clone()),
    );

    let config = RewardConfig::load(&deps.storage).unwrap();
    assert_eq!(config.participation_reward_amount, participation_reward_amount);
    assert_eq!(config.referral_reward_amounts, referral_reward_amounts);
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
        Some(vec![Uint128::zero(), Uint128::from(100u64)]),
    );

    will_success(
        &mut deps,
        Some(Uint128::zero()),
        None,
    );

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        Some(vec![]),
    );
    expect_generic_err(&result, "Invalid reward scheme");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        None,
        Some(vec![Uint128::zero(), Uint128::zero()]),
    );
    expect_generic_err(&result, "Invalid reward scheme");
}
