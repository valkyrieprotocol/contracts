use cosmwasm_std::{Env, MessageInfo, Response, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::governance::execute_msgs::StakingConfigInitMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use crate::staking::executions::instantiate;
use crate::staking::states::{StakingConfig, StakingState};
use crate::tests::{default_env, default_info, WITHDRAW_DELAY};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, withdraw_delay: u64) -> ContractResult<Response> {
    let msg = StakingConfigInitMsg {
        withdraw_delay
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = default_env();
    let info = default_info();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        WITHDRAW_DELAY,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    default(&mut deps);

    // Validate
    let staking_config = StakingConfig::load(&deps.storage).unwrap();
    assert_eq!(staking_config.withdraw_delay, WITHDRAW_DELAY);

    let staking_state = StakingState::load(&deps.storage).unwrap();
    assert_eq!(staking_state.total_share, Uint128::zero())
}
