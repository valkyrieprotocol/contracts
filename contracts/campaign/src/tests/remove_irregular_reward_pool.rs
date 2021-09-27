use cosmwasm_std::{Env, MessageInfo, Response, Addr, coin, Uint128, SubMsg, CosmosMsg, BankMsg, Decimal, WasmMsg, to_binary};

use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::*;
use valkyrie::test_constants::default_sender;
use valkyrie::test_utils::expect_unauthorized_err;

use crate::executions::remove_irregular_reward_pool;
use crate::states::{CampaignState, Balance};
use valkyrie::test_constants::campaign_manager::REFERRAL_REWARD_TOKEN;
use cw20::Cw20ExecuteMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
) -> ContractResult<Response> {
    remove_irregular_reward_pool(deps.as_mut(), env, info, denom)
}

pub fn will_success(deps: &mut CustomDeps, denom: Denom) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    let response = exec(deps, env.clone(), info.clone(), denom).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();
    deps.querier.with_tax(Decimal::percent(10), &[
        ("uusd", &Uint128::new(100)),
        ("ukrw", &Uint128::new(100)),
    ]);

    super::instantiate::default(&mut deps);
    super::add_reward_pool::will_success(&mut deps, 500, 500);

    deps.querier.plus_native_balance(CAMPAIGN, vec![
        coin(1000, "ukrw"),
        coin(1000, PARTICIPATION_REWARD_DENOM_NATIVE),
    ]);
    deps.querier.plus_token_balances(&[(REFERRAL_REWARD_TOKEN, &[
        (CAMPAIGN, &Uint128::new(1000)),
    ])]);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Native("ukrw".to_string())),
        Balance::default(),
    );
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())),
        Balance {
            total: Uint128::new(500),
            locked: Uint128::zero(),
        },
    );
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))),
        Balance {
            total: Uint128::new(500),
            locked: Uint128::zero(),
        },
    );

    let (_, info, response) = will_success(
        &mut deps,
        Denom::Native("ukrw".to_string()),
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(909, "ukrw")],
        }))
    ]);

    let (_, info, response) = will_success(
        &mut deps,
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(909, PARTICIPATION_REWARD_DENOM_NATIVE)],
        }))
    ]);

    let (_, info, response) = will_success(
        &mut deps,
        Denom::Token(REFERRAL_REWARD_TOKEN.to_string()),
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: Uint128::new(1000),
            }).unwrap(),
        })),
    ]);
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
        Denom::Native("uusd".to_string()),
    );
    expect_unauthorized_err(&result);
}
