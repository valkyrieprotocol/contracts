use crate::entrypoints::{execute, instantiate, query};

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{attr, from_binary, to_binary, CosmosMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use valkyrie::distributor::{
    execute_msgs::{ExecuteMsg, InstantiateMsg},
    query_msgs::{ContractConfigResponse, QueryMsg},
};
use valkyrie::errors::ContractError;

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        governance: "gov".to_string(),
        token_contract: "valkyrie".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ContractConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::ContractConfig {}).unwrap())
            .unwrap();
    assert_eq!("gov", config.governance.as_str());
    assert_eq!("valkyrie", config.token_contract.as_str());
}

#[test]
fn test_add_distributor() {
    let mut deps = mock_dependencies(&[]);

    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {
            governance: "gov".to_string(),
            token_contract: "valkyrie".to_string(),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::AddDistributor {
        distributor: "distributor0000".to_string(),
        spend_limit: Uint128::from(10000000000u128),
    };

    // Unauthorized Addition
    let unauthorized_info = mock_info("addr0000", &[]);
    match execute(deps.as_mut(), mock_env(), unauthorized_info, msg.clone()) {
        Err(ContractError::Unauthorized { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // Normal Addition
    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "add_distributor"),
            attr("distributor", "distributor0000"),
            attr("spend_limit", "10000000000"),
        ]
    );

    // Duplicate Addition
    match execute(deps.as_mut(), mock_env(), info, msg) {
        Err(ContractError::AlreadyExists { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn test_remove_distributor() {
    let mut deps = mock_dependencies(&[]);

    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {
            governance: "gov".to_string(),
            token_contract: "valkyrie".to_string(),
        },
    )
    .unwrap();

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("gov", &[]),
        ExecuteMsg::AddDistributor {
            distributor: "distributor0000".to_string(),
            spend_limit: Uint128::from(10000000000u128),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::RemoveDistributor {
        distributor: "distributor0000".to_string(),
    };

    // Unauthorized Remove
    let unauthorized_info = mock_info("addr0000", &[]);
    match execute(deps.as_mut(), mock_env(), unauthorized_info, msg.clone()) {
        Err(ContractError::Unauthorized { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // Normal Remove
    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "remove_distributor"),
            attr("distributor", "distributor0000"),
        ]
    );

    // Duplicate Remove
    match execute(deps.as_mut(), mock_env(), info, msg) {
        Err(ContractError::NotFound { .. }) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn test_spend() {
    let mut deps = mock_dependencies(&[]);

    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {
            governance: "gov".to_string(),
            token_contract: "valkyrie".to_string(),
        },
    )
    .unwrap();

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("gov", &[]),
        ExecuteMsg::AddDistributor {
            distributor: "distributor0000".to_string(),
            spend_limit: Uint128::from(10000000000u128),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    // Distributor Not Found
    let not_found_info = mock_info("distributor0001", &[]);
    match execute(deps.as_mut(), mock_env(), not_found_info, msg.clone()) {
        Err(ContractError::NotFound {}) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // Normal Spend
    let info = mock_info("distributor0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "spend"),
            attr("distributor", "distributor0000"),
            attr("recipient", "addr0000"),
            attr("amount", "1000000"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "valkyrie".to_string(),
            send: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            })
            .unwrap(),
        })]
    );

    // Over Spend Limit
    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(10000000000u128),
    };
    match execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()) {
        Err(ContractError::ExceedLimit {}) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }
}
