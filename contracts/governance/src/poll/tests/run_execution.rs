use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, CosmosMsg, WasmMsg, SubMsg};
use valkyrie::common::{ExecutionMsg, ContractResult};
use crate::poll::executions::run_execution;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use cosmwasm_std::testing::mock_info;
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
    let env = contract_env();
    let info = mock_info(env.contract.address.as_str(), &[]);

    let response = exec(deps, env.clone(), info.clone(), executions).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

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
    let mut deps = custom_deps(&[]);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        vec![],
    );
    expect_unauthorized_err(&result);
}
