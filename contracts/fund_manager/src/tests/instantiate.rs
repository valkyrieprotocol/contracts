use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, Uint128, Api};
use valkyrie::common::ContractResult;
use crate::executions::instantiate;
use valkyrie::fund_manager::execute_msgs::InstantiateMsg;
use cosmwasm_std::testing::mock_env;
use valkyrie::test_utils::default_sender;
use crate::tests::{ADMINS, TOKEN_CONTRACT, TERRASWAP_ROUTER};
use crate::states::{ContractConfig, ContractState};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admins: Vec<String>,
    managing_token: String,
    terraswap_router: String,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        admins,
        managing_token,
        terraswap_router,
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
        ADMINS.iter().map(|v| v.to_string()).collect(),
        TOKEN_CONTRACT.to_string(),
        TERRASWAP_ROUTER.to_string(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    default(&mut deps);

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config, ContractConfig {
        admins: ADMINS.iter().map(|v| deps.api.addr_validate(v).unwrap()).collect(),
        managing_token: Addr::unchecked(TOKEN_CONTRACT),
        terraswap_router: Addr::unchecked(TERRASWAP_ROUTER)
    });

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state, ContractState {
        remain_allowance_amount: Uint128::zero(),
    });
}
