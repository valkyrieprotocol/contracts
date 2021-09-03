use cosmwasm_std::{Env, Response};

use valkyrie::campaign::execute_msgs::MigrateMsg;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{CustomDeps, custom_deps};

use crate::executions::migrate;
use valkyrie::test_constants::campaign::campaign_env;
use crate::states::CampaignState;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
) -> ContractResult<Response> {
    migrate(deps.as_mut(), env, MigrateMsg {})
}

pub fn will_success(deps: &mut CustomDeps, chain_id: &str) -> (Env, Response) {
    let mut env = campaign_env();

    env.block.chain_id = chain_id.to_string();

    let response = exec(deps, env.clone()).unwrap();

    (env, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    will_success(&mut deps, "new-chain-id");

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.chain_id, "new-chain-id".to_string());
}
