use cosmwasm_std::{Addr, Env, Response, to_binary, Uint128};
use cosmwasm_std::testing::mock_info;
use cw20::Cw20ReceiveMsg;
use valkyrie::common::ContractResult;
use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::liquidity::LP_LIQUIDITY_TOKEN;

use crate::entrypoints::{execute};
use crate::tests::instantiate::default;
use crate::states::{StakerInfo, State};

pub fn exec_bond(deps: &mut CustomDeps, env:&Env, sender:&Addr, amount:Uint128) -> ContractResult<Response> {
    let info = mock_info(LP_LIQUIDITY_TOKEN, &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount,
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });

    execute(deps.as_mut(), env.clone(), info.clone(), msg)
}

fn will_success(deps: &mut CustomDeps, env:Env, sender:&Addr) {
    let amount = Uint128::new(100u128);
    exec_bond(deps, &env, sender, amount).unwrap();
}

#[test]
fn succeed() {
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut deps = custom_deps();
    let (env, _info, _response) = default(&mut deps, None);
    will_success(&mut deps, env.clone(), &sender1);
    will_success(&mut deps, env.clone(), &sender2);

    let state1 = State::load(deps.as_ref().storage).unwrap();
    let info1 = StakerInfo::load_or_default(deps.as_ref().storage, &sender1).unwrap();
    let info2 = StakerInfo::load_or_default(deps.as_ref().storage, &sender2).unwrap();

    assert_eq!(state1.total_bond_amount, Uint128::new(200u128));
    assert_eq!(state1.last_distributed, 0);

    assert_eq!(info1.pending_reward, Uint128::zero());
    assert_eq!(info1.bond_amount, Uint128::new(100u128));

    assert_eq!(info2.pending_reward, Uint128::zero());
    assert_eq!(info2.bond_amount, Uint128::new(100u128));
}