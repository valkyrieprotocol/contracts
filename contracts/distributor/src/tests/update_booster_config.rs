use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Decimal, Response};
use valkyrie::common::ContractResult;
use crate::executions::update_booster_config;
use valkyrie::distributor::execute_msgs::BoosterConfig;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::{governance_sender, DROP_BOOSTER_RATIO_PERCENT, ACTIVITY_BOOSTER_RATIO_PERCENT, PLUS_BOOSTER_RATIO_PERCENT, ACTIVITY_BOOSTER_MULTIPLIER_PERCENT};
use crate::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    drop_booster_ratio: Decimal,
    activity_booster_ratio: Decimal,
    plus_booster_ratio: Decimal,
    activity_booster_multiplier: Decimal,
) -> ContractResult<Response> {
    let msg = BoosterConfig {
        drop_booster_ratio,
        activity_booster_ratio,
        plus_booster_ratio,
        activity_booster_multiplier,
    };

    update_booster_config(deps.as_mut(), env, info, msg)
}

pub fn will_success(
    deps: &mut CustomDeps,
    drop_booster_ratio: Decimal,
    activity_booster_ratio: Decimal,
    plus_booster_ratio: Decimal,
    activity_booster_multiplier: Decimal,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        drop_booster_ratio,
        activity_booster_ratio,
        plus_booster_ratio,
        activity_booster_multiplier,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let drop_booster_ratio = Decimal::percent(DROP_BOOSTER_RATIO_PERCENT + 1);
    let activity_booster_ratio = Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT - 2);
    let plus_booster_ratio = Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT + 1);
    let activity_booster_multiplier = Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT + 1);

    will_success(
        &mut deps,
        drop_booster_ratio,
        activity_booster_ratio,
        plus_booster_ratio,
        activity_booster_multiplier,
    );

    let config = ContractConfig::load(&deps.storage).unwrap().booster_config;
    assert_eq!(config, BoosterConfig {
        drop_booster_ratio,
        activity_booster_ratio,
        plus_booster_ratio,
        activity_booster_multiplier,
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
        Decimal::percent(DROP_BOOSTER_RATIO_PERCENT),
        Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT),
        Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT),
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
    );

    expect_unauthorized_err(&result);
}