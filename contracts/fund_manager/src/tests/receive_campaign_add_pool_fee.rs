use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{MessageInfo, Env, Uint128, Response};
use valkyrie::common::ContractResult;
use crate::executions::receive_campaign_add_pool_fee;
use valkyrie::test_constants::fund_manager::fund_manager_env;
use valkyrie::test_constants::campaign::campaign_sender;
use crate::states::ContractState;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    receive_campaign_add_pool_fee(deps.as_mut(), env, info, amount)
}

pub fn will_success(deps: &mut CustomDeps, amount: Uint128) -> (Env, MessageInfo, Response) {
    let env = fund_manager_env();
    let info = campaign_sender();

    let response = exec(deps, env.clone(), info.clone(), amount).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(&mut deps, Uint128::new(33));

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.campaign_add_pool_fee_amount, Uint128::new(33));

    will_success(&mut deps, Uint128::new(20));

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.campaign_add_pool_fee_amount, Uint128::new(53));
}
