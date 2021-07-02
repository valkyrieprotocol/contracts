use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Decimal, Response, Addr};
use valkyrie::common::ContractResult;
use valkyrie::distributor::execute_msgs::{InstantiateMsg, BoosterConfig};
use crate::executions::instantiate;
use cosmwasm_std::testing::mock_env;
use valkyrie::test_utils::{default_sender, expect_generic_err};
use crate::tests::{GOVERNANCE, TOKEN_CONTRACT, DROP_BOOSTER_RATIO_PERCENT, ACTIVITY_BOOSTER_RATIO_PERCENT, PLUS_BOOSTER_RATIO_PERCENT, ACTIVITY_BOOSTER_MULTIPLIER_PERCENT};
use crate::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: String,
    token_contract: String,
    drop_booster_ratio: Decimal,
    activity_booster_ratio: Decimal,
    plus_booster_ratio: Decimal,
    activity_booster_multiplier: Decimal,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        governance,
        token_contract,
        booster_config: BoosterConfig {
            drop_booster_ratio,
            activity_booster_ratio,
            plus_booster_ratio,
            activity_booster_multiplier,
        },
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        GOVERNANCE.to_string(),
        TOKEN_CONTRACT.to_string(),
        Decimal::percent(DROP_BOOSTER_RATIO_PERCENT),
        Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT),
        Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT),
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    default(&mut deps);

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config, ContractConfig {
        governance: Addr::unchecked(GOVERNANCE),
        token_contract: Addr::unchecked(TOKEN_CONTRACT),
        booster_config: BoosterConfig {
            drop_booster_ratio: Decimal::percent(DROP_BOOSTER_RATIO_PERCENT),
            activity_booster_ratio: Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT),
            plus_booster_ratio: Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT),
            activity_booster_multiplier: Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
        },
    });
}

#[test]
fn failed_invalid_boost_config() {
    let mut deps = custom_deps(&[]);

    let result = exec(
        &mut deps,
        mock_env(),
        default_sender(),
        GOVERNANCE.to_string(),
        TOKEN_CONTRACT.to_string(),
        Decimal::percent(10),
        Decimal::percent(79),
        Decimal::percent(10),
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
    );

    expect_generic_err(&result, "invalid boost_config");
}