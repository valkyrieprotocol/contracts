use cosmwasm_std::{BankMsg, coin, CosmosMsg, Decimal, Env, MessageInfo, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::common::{ContractResult, Denom};
use valkyrie::message_matchers;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{default_sender, VALKYRIE_TOKEN};
use valkyrie::test_constants::campaign::{CAMPAIGN, CAMPAIGN_ADMIN, campaign_admin_sender, campaign_env, PARTICIPATION_REWARD_AMOUNT, PARTICIPATION_REWARD_DENOM_NATIVE, PARTICIPATOR1, PARTICIPATOR2, REFERRAL_REWARD_AMOUNTS};
use valkyrie::test_constants::campaign_manager::CAMPAIGN_MANAGER;
use valkyrie::test_utils::{expect_generic_err, expect_unauthorized_err};
use valkyrie::utils::calc_ratio_amount;

use crate::executions::remove_reward_pool;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let response = remove_reward_pool(deps.as_mut(), env, info, denom, amount)?;

    for msg in message_matchers::native_send(&response.messages) {
        deps.querier.minus_native_balance(CAMPAIGN, msg.amount.clone());
        deps.querier.plus_native_balance(msg.to_address.as_str(), msg.amount);
    }

    for msg in message_matchers::cw20_transfer(&response.messages) {
        deps.querier.minus_token_balances(&[(
            &msg.contract_addr,
            &[(CAMPAIGN, &msg.amount)],
        )]);
        deps.querier.plus_token_balances(&[(
            &msg.contract_addr,
            &[(&msg.recipient, &msg.amount)],
        )]);
    }

    Ok(response)
}

pub fn will_success(
    deps: &mut CustomDeps,
    denom: Denom,
    amount: Option<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = campaign_admin_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        denom,
        amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed_at_pending() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::add_reward_pool::will_success(&mut deps, 10000, 10000);

    let mut denom = Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string());
    let amount = Uint128::new(4000);

    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(amount.u128(), "uluna")],
        })),
    ]);

    let remain_amount = Uint128::new(6000);
    let (_, _, response) = will_success(&mut deps, denom.clone(), None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(remain_amount.u128(), "uluna")],
        })),
    ]);

    denom = Denom::Token(VALKYRIE_TOKEN.to_string());

    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: CAMPAIGN_ADMIN.to_string(),
                amount,
            }).unwrap(),
            funds: vec![],
        })),
    ]);

    let (_, _, response) = will_success(&mut deps, denom.clone(), None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: CAMPAIGN_ADMIN.to_string(),
                amount: remain_amount,
            }).unwrap(),
            funds: vec![],
        })),
    ]);
}

#[test]
fn succeed_at_active() {
    let mut deps = custom_deps();

    let burn_rate = Decimal::percent(10);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 10000, 10000);

    let mut denom = Denom::Native("uluna".to_string());
    let amount = Uint128::new(4000);
    let (burn_amount, expect_amount) = calc_ratio_amount(amount, burn_rate);

    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_MANAGER.to_string(),
            amount: vec![coin(burn_amount.u128(), "uluna")],
        })),
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(expect_amount.u128(), "uluna")],
        })),
    ]);

    let remain_amount = Uint128::new(6000);
    let (burn_amount, expect_amount) = calc_ratio_amount(remain_amount, burn_rate);

    let (_, _, response) = will_success(&mut deps, denom.clone(), None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_MANAGER.to_string(),
            amount: vec![coin(burn_amount.u128(), "uluna")],
        })),
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(expect_amount.u128(), "uluna")],
        })),
    ]);

    denom = Denom::Token(VALKYRIE_TOKEN.to_string());

    let (burn_amount, expect_amount) = calc_ratio_amount(amount, burn_rate);
    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: CAMPAIGN_MANAGER.to_string(),
                amount: burn_amount,
            }).unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: CAMPAIGN_ADMIN.to_string(),
                amount: expect_amount,
            }).unwrap(),
            funds: vec![],
        })),
    ]);

    let (burn_amount, expect_amount) = calc_ratio_amount(remain_amount, burn_rate);
    let (_, _, response) = will_success(&mut deps, denom.clone(), None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: CAMPAIGN_MANAGER.to_string(),
                amount: burn_amount,
            }).unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: CAMPAIGN_ADMIN.to_string(),
                amount: expect_amount,
            }).unwrap(),
            funds: vec![],
        })),
    ]);
}

#[test]
fn succeed_free_balance() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 7000, 10000);
    super::participate::will_success(&mut deps, PARTICIPATOR1, None);
    super::participate::will_success(
        &mut deps,
        PARTICIPATOR2,
        Some(Referrer::Address(PARTICIPATOR1.to_string())),
    );

    will_success(
        &mut deps,
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        Some(
            Uint128::new(7000)
                .checked_sub(PARTICIPATION_REWARD_AMOUNT).unwrap()
                .checked_sub(PARTICIPATION_REWARD_AMOUNT).unwrap(),
        ),
    );
    will_success(
        &mut deps,
        Denom::Token(VALKYRIE_TOKEN.to_string()),
        Some(REFERRAL_REWARD_AMOUNTS[0]),
    );
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
        Denom::Native("uluna".to_string()),
        None,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::add_reward_pool::will_success(&mut deps, 6000, 2000);

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        Denom::Native("uluna".to_string()),
        Some(Uint128::new(6001)),
    );
    expect_generic_err(&result, "Insufficient balance");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        Denom::Token(VALKYRIE_TOKEN.to_string()),
        Some(Uint128::new(2001)),
    );
    expect_generic_err(&result, "Insufficient balance");

    super::update_activation::will_success(&mut deps, true);

    super::participate::will_success(&mut deps, PARTICIPATOR1, None);
    super::participate::will_success(
        &mut deps,
        PARTICIPATOR2,
        Some(Referrer::Address(PARTICIPATOR1.to_string())),
    );

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()),
        Some(
            Uint128::new(6000)
                .checked_sub(PARTICIPATION_REWARD_AMOUNT).unwrap()
                .checked_sub(PARTICIPATION_REWARD_AMOUNT).unwrap()
                .checked_add(Uint128::new(1)).unwrap(),
        ),
    );
    expect_generic_err(&result, "Insufficient balance");

    let result = exec(
        &mut deps,
        campaign_env(),
        campaign_admin_sender(),
        Denom::Token(VALKYRIE_TOKEN.to_string()),
        Some(
            Uint128::new(2000)
                .checked_sub(REFERRAL_REWARD_AMOUNTS[0]).unwrap()
                .checked_add(Uint128::new(1)).unwrap()
                .checked_add(Uint128::new(1600)).unwrap(),
        ),
    );
    expect_generic_err(&result, "Insufficient balance");
}
