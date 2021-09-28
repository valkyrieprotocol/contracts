use cosmwasm_std::{Addr, Api, Env, MessageInfo, Response, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::fund_manager::execute_msgs::InstantiateMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::fund_manager::{ADMINS, fund_manager_env, MANAGING_TOKEN};

use crate::executions::instantiate;
use crate::states::{ContractConfig, ContractState};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admins: Vec<String>,
    managing_token: String,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        admins,
        managing_token,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = fund_manager_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        ADMINS.iter().map(|v| v.to_string()).collect(),
        MANAGING_TOKEN.to_string(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config, ContractConfig {
        admins: ADMINS.iter().map(|v| deps.api.addr_validate(v).unwrap()).collect(),
        managing_token: Addr::unchecked(MANAGING_TOKEN),
    });

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state, ContractState {
        remain_allowance_amount: Uint128::zero(),
    });
}
