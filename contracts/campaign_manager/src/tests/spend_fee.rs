use cosmwasm_std::{Env, MessageInfo, Response, Uint128, SubMsg, CosmosMsg, WasmMsg, to_binary, Decimal};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{default_sender, VALKYRIE_TOKEN};

use crate::executions::spend_fee;
use cw20::Cw20ExecuteMsg;
use valkyrie::test_constants::campaign_manager::{campaign_manager_env, FEE_RECIPIENT, CAMPAIGN_MANAGER};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    spend_fee(deps.as_mut(), env, info, amount)
}

pub fn will_success(deps: &mut CustomDeps, amount: Option<Uint128>) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = default_sender();

    let response = exec(deps, env.clone(), info.clone(), amount).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    deps.querier.plus_token_balances(&[
        (VALKYRIE_TOKEN, &[
            (CAMPAIGN_MANAGER, &Uint128::new(50))
        ])
    ]);

    let (_, _, response) = will_success(&mut deps, None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: FEE_RECIPIENT.to_string(),
                amount: Uint128::new(25),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(25),
            }).unwrap(),
        })),
    ]);

    super::update_config::will_success(
        &mut deps,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Decimal::percent(10)),
        None,
        None,
        None,
        None,
    );

    deps.querier.plus_token_balances(&[
        (VALKYRIE_TOKEN, &[
            (CAMPAIGN_MANAGER, &Uint128::new(100))
        ])
    ]);

    let (_, _, response) = will_success(&mut deps, Some(Uint128::new(50)));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: FEE_RECIPIENT.to_string(),
                amount: Uint128::new(45),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(5),
            }).unwrap(),
        })),
    ]);
}
