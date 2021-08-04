use cosmwasm_std::{CosmosMsg, Env, MessageInfo, Response, SubMsg, WasmMsg};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::{ContractResult, ExecutionMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::governance_env;
use valkyrie::test_utils::expect_unauthorized_err;

use crate::poll::executions::run_execution;
use crate::poll::tests::create_poll::mock_exec_msg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    run_execution(deps.as_mut(), env, info, executions)
}

pub fn will_success(
    deps: &mut CustomDeps,
    executions: Vec<ExecutionMsg>,
) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = mock_info(env.contract.address.as_str(), &[]);

    let response = exec(deps, env.clone(), info.clone(), executions).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    let executions = vec![
        mock_exec_msg(1),
        mock_exec_msg(2),
    ];

    let (_, _, response) = will_success(&mut deps, executions.clone());
    assert_eq!(response.messages, executions.iter().map(|e| SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: e.contract.to_string(),
        funds: vec![],
        msg: e.msg.clone(),
    }))).collect::<Vec<SubMsg>>())
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        governance_env(),
        default_sender(),
        vec![],
    );
    expect_unauthorized_err(&result);
}
