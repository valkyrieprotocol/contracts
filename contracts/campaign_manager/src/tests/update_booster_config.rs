use cosmwasm_std::{Addr, Decimal, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_generic_err};

use crate::executions::update_booster_config;
use crate::states::BoosterConfig;
use crate::tests::{ACTIVITY_BOOSTER_RATIO_PERCENT, DROP_BOOSTER_RATIO_PERCENT, governance_sender, MIN_PARTICIPATION_COUNT, PLUS_BOOSTER_RATIO_PERCENT};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    booster_token: Option<String>,
    drop_booster_ratio: Option<Decimal>,
    activity_booster_ratio: Option<Decimal>,
    plus_booster_ratio: Option<Decimal>,
    activity_booster_multiplier: Option<Decimal>,
    min_participation_count: Option<u64>,
) -> ContractResult<Response> {
    update_booster_config(
        deps.as_mut(),
        env,
        info,
        booster_token,
        drop_booster_ratio,
        activity_booster_ratio,
        plus_booster_ratio,
        activity_booster_multiplier,
        min_participation_count,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    booster_token: Option<String>,
    drop_booster_ratio: Option<Decimal>,
    activity_booster_ratio: Option<Decimal>,
    plus_booster_ratio: Option<Decimal>,
    activity_booster_multiplier: Option<Decimal>,
    min_participation_count: Option<u64>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        booster_token,
        drop_booster_ratio,
        activity_booster_ratio,
        plus_booster_ratio,
        activity_booster_multiplier,
        min_participation_count,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let booster_token = "ChangedBoosterToken";
    let drop_booster_ratio = Decimal::percent(DROP_BOOSTER_RATIO_PERCENT + 1);
    let activity_booster_ratio = Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT - 2);
    let plus_booster_ratio = Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT + 1);
    let activity_booster_multiplier = Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT + 1);
    let min_participation_count = MIN_PARTICIPATION_COUNT + 1;

    will_success(
        &mut deps,
        Some(booster_token.to_string()),
        Some(drop_booster_ratio),
        Some(activity_booster_ratio),
        Some(plus_booster_ratio),
        Some(activity_booster_multiplier),
        Some(min_participation_count),
    );

    let config = BoosterConfig::load(&deps.storage).unwrap();
    assert_eq!(config, BoosterConfig {
        booster_token: Addr::unchecked(booster_token),
        drop_ratio: drop_booster_ratio,
        activity_ratio: activity_booster_ratio,
        plus_ratio: plus_booster_ratio,
        activity_multiplier: activity_booster_multiplier,
        min_participation_count,
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
        None,
        None,
        None,
        None,
        None,
        None,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_invalid_booster_ratio() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        governance_sender(),
        None,
        None,
        Some(Decimal::percent(81)),
        None,
        None,
        None,
    );

    expect_generic_err(&result, "Invalid booster ratio");
}
