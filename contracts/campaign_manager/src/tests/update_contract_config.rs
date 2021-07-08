use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr};
use valkyrie::common::ContractResult;
use crate::executions::update_contract_config;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::governance_sender;
use crate::states::ContractConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: Option<String>,
    fund_manager: Option<String>,
) -> ContractResult<Response> {
    update_contract_config(
        deps.as_mut(),
        env,
        info,
        governance,
        fund_manager,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    governance: Option<String>,
    fund_manager: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        governance,
        fund_manager,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let governance = "ChangedGovernance";
    let fund_manager = "ChangedFundManager";

    will_success(&mut deps, Some(governance.to_string()), Some(fund_manager.to_string()));

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config, ContractConfig {
        governance: Addr::unchecked(governance),
        fund_manager: Addr::unchecked(fund_manager),
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        None,
        None,
    );
    expect_unauthorized_err(&result);
}
