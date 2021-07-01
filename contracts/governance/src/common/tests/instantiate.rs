use cosmwasm_std::{Env, MessageInfo, Response};
use cosmwasm_std::testing::mock_env;

use valkyrie::common::ContractResult;
use valkyrie::governance::execute_msgs::ContractConfigInitMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::default_sender;

use crate::common::executions;
use crate::common::states::ContractConfig;
use crate::tests::TOKEN_CONTRACT;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo, token_contract: String) -> ContractResult<Response> {
    let msg = ContractConfigInitMsg {
        token_contract,
    };

    // Execute
    executions::instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        TOKEN_CONTRACT.to_string(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    // Initialize
    let mut deps = custom_deps(&[]);

    let (env, _, _) = default(&mut deps);

    // Validate
    let contract_config = ContractConfig::load(&deps.storage).unwrap();

    assert_eq!(TOKEN_CONTRACT, contract_config.token_contract.as_str());
    assert_eq!(env.contract.address, contract_config.address);
}