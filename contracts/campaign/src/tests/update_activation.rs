use cosmwasm_std::{Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::{campaign_admin_sender, campaign_env, campaign_env_height, QUALIFIER};
use valkyrie::test_constants::default_sender;
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};

use crate::executions::{update_activation, update_campaign_config};
use crate::states::CampaignState;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    is_active: bool,
) -> ContractResult<Response> {
    update_activation(deps.as_mut(), env, info, is_active)
}

pub fn will_success(deps: &mut CustomDeps, is_active: bool) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    update_campaign_config(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(QUALIFIER.to_string()),
        Some("descdesc".to_string()),
        None,
    ).unwrap();

    let response = exec(deps, env.clone(), info.clone(), is_active).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let (env, _, _) = will_success(&mut deps, true);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.active_flag, true);
    assert_eq!(campaign_state.last_active_height, Some(env.block.height));

    let deactivate_env = campaign_env_height(env.block.height + 1);
    exec(&mut deps, deactivate_env, campaign_admin_sender(), false).unwrap();

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.active_flag, false);
    assert_eq!(campaign_state.last_active_height, Some(env.block.height));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
        true,
    );

    expect_unauthorized_err(&result);
}


#[test]
fn failed_no_qualifier() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        true,
    );

    expect_generic_err(&result, "Required set qualifier");
}
