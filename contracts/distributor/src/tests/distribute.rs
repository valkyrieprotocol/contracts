use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Uint128, SubMsg, CosmosMsg, WasmMsg, to_binary};
use valkyrie::common::ContractResult;
use crate::executions::distribute;
use valkyrie::test_utils::set_height;
use crate::states::{Distribution, ContractState};
use valkyrie::test_constants::distributor::{distributor_env, MANAGING_TOKEN, DISTRIBUTOR};
use valkyrie::test_constants::governance::governance_sender;
use cw20::Cw20ExecuteMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    id: Option<u64>,
) -> ContractResult<Response> {
    distribute(
        deps.as_mut(),
        env,
        info,
        id,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    height: u64,
    id: Option<u64>,
) -> (Env, MessageInfo, Response) {
    let mut env = distributor_env();
    set_height(&mut env, height);

    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        id,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(15000)),
    ])]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
    );
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient2".to_string(),
        Uint128::new(5000),
    );

    let (_, _, response) = will_success(&mut deps, 20001, None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MANAGING_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "Recipient".to_string(),
                amount: Uint128::new(1),
            }).unwrap(),
        })),
    ]);

    let (_, _, response) = will_success(&mut deps, 20002, None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MANAGING_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "Recipient".to_string(),
                amount: Uint128::new(1),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MANAGING_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "Recipient2".to_string(),
                amount: Uint128::new(1),
            }).unwrap(),
        })),
    ]);

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state.distribution_count, 2);
    assert_eq!(state.locked_amount, Uint128::new(14997));
    assert_eq!(state.distributed_amount, Uint128::new(3));

    let distribution = Distribution::may_load(&deps.storage, 1).unwrap().unwrap();
    assert_eq!(distribution.distributed_amount, Uint128::new(2));

    let distribution = Distribution::may_load(&deps.storage, 2).unwrap().unwrap();
    assert_eq!(distribution.distributed_amount, Uint128::new(1));
}

#[test]
fn validate_released_amount() {
    let mut deps = custom_deps();

    deps.querier.plus_token_balances(&[(MANAGING_TOKEN, &[
        (DISTRIBUTOR, &Uint128::new(15000)),
    ])]);

    super::instantiate::default(&mut deps);
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient".to_string(),
        Uint128::new(10000),
    );
    super::register_distribution::will_success(
        &mut deps,
        20000,
        30000,
        "Recipient2".to_string(),
        Uint128::new(5000),
    );

    let distribution = Distribution::may_load(&deps.storage, 1).unwrap().unwrap();
    assert_eq!(distribution.released_amount(2000), Uint128::zero());
    assert_eq!(distribution.released_amount(20000), Uint128::zero());
    assert_eq!(distribution.released_amount(20001), Uint128::new(1));
    assert_eq!(distribution.released_amount(20010), Uint128::new(10));

    let distribution = Distribution::may_load(&deps.storage, 2).unwrap().unwrap();
    assert_eq!(distribution.released_amount(2000), Uint128::zero());
    assert_eq!(distribution.released_amount(20000), Uint128::zero());
    assert_eq!(distribution.released_amount(20001), Uint128::new(0));
    assert_eq!(distribution.released_amount(20002), Uint128::new(1));
    assert_eq!(distribution.released_amount(20010), Uint128::new(5));
}
