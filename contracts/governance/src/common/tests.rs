use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use valkyrie::governance::messages::ContractConfigInitMsg;
use cosmwasm_std::Addr;
use crate::common::executions;
use crate::common::states::ContractConfig;

#[test]
fn instantiate() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("", &[]);
    let msg = ContractConfigInitMsg {
        token_contract: String::from("TokenContract"),
    };

    // Execute
    executions::instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    // Validate
    let contract_config = ContractConfig::load(&deps.storage).unwrap();

    assert_eq!(msg.token_contract, contract_config.token_contract.as_str());
    assert_eq!(env.contract.address, contract_config.address);
}