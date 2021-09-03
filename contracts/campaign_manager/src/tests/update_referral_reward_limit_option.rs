use cosmwasm_std::{Addr, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::campaign_manager_env;
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_utils::expect_unauthorized_err;

use crate::executions::update_referral_reward_limit_option;
use crate::states::ReferralRewardLimitOption;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    overflow_amount_recipient: Option<String>,
    base_count: Option<u8>,
    percent_for_governance_staking: Option<u16>,
) -> ContractResult<Response> {
    update_referral_reward_limit_option(
        deps.as_mut(),
        env,
        info,
        overflow_amount_recipient,
        base_count,
        percent_for_governance_staking,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    overflow_amount_recipient: Option<String>,
    base_count: Option<u8>,
    percent_for_governance_staking: Option<u16>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        overflow_amount_recipient,
        base_count,
        percent_for_governance_staking,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let overflow_amount_recipient = "ChangedRecipient";
    let base_count = 1u8;
    let percent_for_governance_staking = 10u16;

    will_success(
        &mut deps,
        Some(overflow_amount_recipient.to_string()),
        Some(base_count),
        Some(percent_for_governance_staking),
    );

    let option = ReferralRewardLimitOption::load(&deps.storage).unwrap();
    assert_eq!(option, ReferralRewardLimitOption {
        overflow_amount_recipient: Some(Addr::unchecked(overflow_amount_recipient)),
        base_count,
        percent_for_governance_staking,
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
        None,
        None,
        None,
    );
    expect_unauthorized_err(&result);
}
