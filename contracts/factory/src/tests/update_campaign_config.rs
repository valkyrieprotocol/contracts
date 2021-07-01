use cosmwasm_std::{Decimal, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};

use crate::executions::update_campaign_config;
use crate::states::CampaignConfig;
use crate::tests::{CAMPAIGN_DEACTIVATE_PERIOD, governance_sender, REWARD_WITHDRAW_BURN_RATE_PERCENT};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    reward_withdraw_burn_rate: Option<Decimal>,
    campaign_deactivate_period: Option<u64>,
) -> ContractResult<Response> {
    update_campaign_config(
        deps.as_mut(),
        env,
        info,
        reward_withdraw_burn_rate,
        campaign_deactivate_period,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    reward_withdraw_burn_rate: Option<Decimal>,
    campaign_deactivate_period: Option<u64>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        reward_withdraw_burn_rate,
        campaign_deactivate_period,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let reward_withdraw_burn_rate = Decimal::percent(REWARD_WITHDRAW_BURN_RATE_PERCENT + 1);
    let campaign_deactivate_period = CAMPAIGN_DEACTIVATE_PERIOD + 1;

    will_success(
        &mut deps,
        Some(reward_withdraw_burn_rate),
        Some(campaign_deactivate_period),
    );

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config.reward_withdraw_burn_rate, reward_withdraw_burn_rate);
    assert_ne!(campaign_config.reward_withdraw_burn_rate, Decimal::percent(REWARD_WITHDRAW_BURN_RATE_PERCENT));
    assert_eq!(campaign_config.campaign_deactivate_period, campaign_deactivate_period);
    assert_ne!(campaign_config.campaign_deactivate_period, CAMPAIGN_DEACTIVATE_PERIOD);
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
    );

    expect_unauthorized_err(&result);
}