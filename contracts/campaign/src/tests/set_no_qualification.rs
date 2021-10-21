use cosmwasm_std::{Env, MessageInfo, Response, Addr};

use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::*;
use valkyrie::test_constants::default_sender;
use valkyrie::test_utils::expect_unauthorized_err;

use crate::executions::set_no_qualification;
use crate::states::CampaignConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    set_no_qualification(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::will_success(
        &mut deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        Some("Qualifier".to_string()),
        None,
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        PARTICIPATION_REWARD_AMOUNT,
        PARTICIPATION_REWARD_LOCK_PERIOD,
        REFERRAL_REWARD_AMOUNTS.to_vec(),
        REFERRAL_REWARD_LOCK_PERIOD,
    );

    let config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(config.qualifier, Some(Addr::unchecked("Qualifier")));

    will_success(&mut deps);

    let config = CampaignConfig::load(&deps.storage).unwrap();
    assert!(config.qualifier.is_none());
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(&mut deps, campaign_env(), default_sender());
    expect_unauthorized_err(&result);
}
