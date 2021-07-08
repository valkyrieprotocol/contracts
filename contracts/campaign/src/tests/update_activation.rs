use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response};
use valkyrie::common::ContractResult;
use crate::executions::update_activation;
use valkyrie::test_utils::{contract_env, contract_env_height, default_sender, expect_unauthorized_err};
use crate::tests::campaign_admin_sender;
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
    let env = contract_env();
    let info = campaign_admin_sender();

    let response = exec(deps, env.clone(), info.clone(), is_active).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let (env, _, _) = will_success(&mut deps, true);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.active_flag, true);
    assert_eq!(campaign_state.last_active_height, Some(env.block.height));

    let deactivate_env = contract_env_height(env.block.height + 1);
    exec(&mut deps, deactivate_env, campaign_admin_sender(), false).unwrap();

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.active_flag, false);
    assert_eq!(campaign_state.last_active_height, Some(env.block.height));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        true,
    );
    expect_unauthorized_err(&result);
}
