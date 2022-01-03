use cosmwasm_std::{Addr, Decimal, Env, MessageInfo, Response, to_binary, Uint128};
use cw20::Cw20ReceiveMsg;

use valkyrie::common::ContractResult;
use valkyrie::lp_staking::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{default_sender, DEFAULT_SENDER};
use valkyrie::test_constants::liquidity::*;
use valkyrie::test_utils::expect_generic_err;
use crate::entrypoints::{execute, instantiate};
use crate::states::{Config, State};


pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    token: String,
    pair: String,
    lp_token: String,
    whitelisted_contracts: Vec<String>,
    distribution_schedule: Vec<(u64, u64, Uint128)>,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        token,
        pair,
        lp_token,
        whitelisted_contracts,
        distribution_schedule,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps, total_bonded:Option<Uint128>) -> (Env, MessageInfo, Response) {
    let env = lp_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        LP_REWARD_TOKEN.to_string(),
        LP_PAIR_TOKEN.to_string(),
        LP_LIQUIDITY_TOKEN.to_string(),
        vec![LP_WHITELISTED1.to_string(), LP_WHITELISTED2.to_string()],
        vec![LP_DISTRIBUTION_SCHEDULE1, LP_DISTRIBUTION_SCHEDULE2],
    ).unwrap();

    if let Some(total_bonded) = total_bonded {
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: info.sender.to_string(),
            amount: total_bonded,
            msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
        });

        let mut info = info.clone();
        info.sender = Addr::unchecked(LP_LIQUIDITY_TOKEN);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }

    deps.querier.plus_token_balances(&[(
        LP_REWARD_TOKEN,
        &[(DEFAULT_SENDER, &LP_DISTRIBUTION_SCHEDULE1.2)],
    ), (
        LP_REWARD_TOKEN,
        &[(DEFAULT_SENDER, &LP_DISTRIBUTION_SCHEDULE2.2)],
    )]);

    (env, info, response)
}


#[test]
fn succeed() {
    let mut deps = custom_deps();

    let (env, info, _response) = default(&mut deps, None);

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config.token, LP_REWARD_TOKEN);
    assert_eq!(config.pair, LP_PAIR_TOKEN);
    assert_eq!(config.lp_token, LP_LIQUIDITY_TOKEN);
    assert_eq!(config.admin, info.sender);
    assert_eq!(config.whitelisted_contracts, vec![LP_WHITELISTED1.to_string(), LP_WHITELISTED2.to_string()]);
    assert_eq!(config.distribution_schedule, vec![LP_DISTRIBUTION_SCHEDULE1, LP_DISTRIBUTION_SCHEDULE2]);

    let state = State::load(&deps.storage).unwrap();
    assert_eq!(state.global_reward_index, Decimal::zero());
    assert_eq!(state.last_distributed, env.block.height);
    assert_eq!(state.total_bond_amount, Uint128::zero());
}

#[test]
fn failed_invalid_schedule() {
    let mut deps = custom_deps();

    let result = exec(
        &mut deps,
        lp_env(),
        default_sender(),
        LP_REWARD_TOKEN.to_string(),
        LP_PAIR_TOKEN.to_string(),
        LP_LIQUIDITY_TOKEN.to_string(),
        vec![LP_WHITELISTED1.to_string(), LP_WHITELISTED2.to_string()],
        vec![(100,100, Uint128::new(100u128))]
    );

    expect_generic_err(&result, "invalid schedule");
}