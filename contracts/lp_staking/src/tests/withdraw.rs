use cosmwasm_std::{Addr, CosmosMsg, Env, MessageInfo, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use valkyrie::common::ContractResult;

use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::DEFAULT_SENDER;
use valkyrie::test_constants::liquidity::LP_REWARD_TOKEN;
use crate::executions::withdraw;
use crate::tests::instantiate::default;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,

) -> ContractResult<Response> {
    withdraw(
        deps.as_mut(),
        env,
        info,
    )
}

fn will_success(deps: &mut CustomDeps) -> Response {
    let (mut env, info, _response) = default(deps, Some(Uint128::new(100u128)));
    env.block.height = 20;
    exec(deps, env, info).unwrap()
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    let res = will_success(&mut deps);

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: Addr::unchecked(LP_REWARD_TOKEN).to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: Addr::unchecked(DEFAULT_SENDER).to_string(),
                amount: Uint128::new(200000u128),
            })
                .unwrap(),
            funds: vec![],
        }))]
    );
}
