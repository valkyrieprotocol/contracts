use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, coin, CosmosMsg, BankMsg, WasmMsg, to_binary, Decimal, SubMsg};
use valkyrie::common::{ContractResult, Denom};
use crate::executions::withdraw;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err, expect_generic_err};
use crate::tests::{campaign_admin_sender, TOKEN_CONTRACT, CAMPAIGN_ADMIN, CAMPAIGN_DISTRIBUTION_AMOUNTS, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE, FUND_MANAGER};
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use valkyrie::terra::extract_tax;
use cw20::Cw20ExecuteMsg;
use valkyrie::utils::calc_ratio_amount;
use valkyrie::message_matchers;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let response = withdraw(deps.as_mut(), env, info, denom, amount)?;

    for msg in message_matchers::native_send(&response.messages) {
        deps.querier.minus_native_balance_with_tax(MOCK_CONTRACT_ADDR, msg.amount.clone());
        deps.querier.plus_native_balance(msg.to_address.as_str(), msg.amount);
    }

    for msg in message_matchers::cw20_transfer(&response.messages) {
        deps.querier.minus_token_balances(&[(
            &msg.contract_addr,
            &[(MOCK_CONTRACT_ADDR, &msg.amount)],
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
    let env = contract_env();
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
    let mut deps = custom_deps(&[coin(10000u128, "uusd")]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128::new(10000))],
    )]);
    deps.querier.with_tax(Decimal::percent(10), &[("uusd", &Uint128::new(100))]);

    super::instantiate::default(&mut deps);

    let mut denom = Denom::Native("uusd".to_string());
    let amount = Uint128::new(4000);
    let tax = extract_tax(&deps.as_ref().querier, "uusd".to_string(), amount).unwrap();

    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(amount.checked_sub(tax).unwrap().u128(), "uusd")],
        })),
    ]);

    let remain_amount = Uint128::new(6000);
    let tax = extract_tax(&deps.as_ref().querier, "uusd".to_string(), remain_amount).unwrap();
    let (_, _, response) = will_success(&mut deps, denom.clone(), None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(remain_amount.checked_sub(tax).unwrap().u128(), "uusd")],
        })),
    ]);

    denom = Denom::Token(TOKEN_CONTRACT.to_string());

    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
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
            contract_addr: TOKEN_CONTRACT.to_string(),
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
    let mut deps = custom_deps(&[coin(10000u128, "uusd")]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128::new(10000))],
    )]);
    deps.querier.with_tax(Decimal::percent(10), &[("uusd", &Uint128::new(100))]);

    let burn_rate = Decimal::percent(10);
    deps.querier.with_global_campaign_config(
        TOKEN_CONTRACT.to_string(),
        Uint128::new(1000),
        FUND_MANAGER.to_string(),
        1,
        vec![Denom::Token(TOKEN_CONTRACT.to_string()), Denom::Native("uusd".to_string())],
        burn_rate.clone(),
        FUND_MANAGER.to_string(),
        1000,
    );

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    let mut denom = Denom::Native("uusd".to_string());
    let amount = Uint128::new(4000);
    let (burn_amount, expect_amount) = calc_ratio_amount(amount, burn_rate);
    let burn_tax = extract_tax(&deps.as_ref().querier, "uusd".to_string(), burn_amount).unwrap();
    let expect_tax = extract_tax(&deps.as_ref().querier, "uusd".to_string(), expect_amount).unwrap();

    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: FUND_MANAGER.to_string(),
            amount: vec![coin(burn_amount.checked_sub(burn_tax).unwrap().u128(), "uusd")],
        })),
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(expect_amount.checked_sub(expect_tax).unwrap().u128(), "uusd")],
        })),
    ]);

    let remain_amount = Uint128::new(6000);
    let (burn_amount, expect_amount) = calc_ratio_amount(remain_amount, burn_rate);
    let burn_tax = extract_tax(&deps.as_ref().querier, "uusd".to_string(), burn_amount).unwrap();
    let expect_tax = extract_tax(&deps.as_ref().querier, "uusd".to_string(), expect_amount).unwrap();
    let (_, _, response) = will_success(&mut deps, denom.clone(), None);
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: FUND_MANAGER.to_string(),
            amount: vec![coin(burn_amount.checked_sub(burn_tax).unwrap().u128(), "uusd")],
        })),
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: CAMPAIGN_ADMIN.to_string(),
            amount: vec![coin(expect_amount.checked_sub(expect_tax).unwrap().u128(), "uusd")],
        })),
    ]);

    denom = Denom::Token(TOKEN_CONTRACT.to_string());

    let (burn_amount, expect_amount) = calc_ratio_amount(amount, burn_rate);
    let (_, _, response) = will_success(&mut deps, denom.clone(), Some(amount));
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: FUND_MANAGER.to_string(),
                amount: burn_amount,
            }).unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
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
            contract_addr: TOKEN_CONTRACT.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: FUND_MANAGER.to_string(),
                amount: burn_amount,
            }).unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TOKEN_CONTRACT.to_string(),
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
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Participator1", None);

    will_success(
        &mut deps,
        Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        Some(Uint128::new(1000).checked_sub(CAMPAIGN_DISTRIBUTION_AMOUNTS[0]).unwrap()),
    );
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        Denom::Native("uusd".to_string()),
        None,
    );

    expect_unauthorized_err(&result);
}

#[test]
fn failed_overflow() {
    let mut deps = custom_deps(&[coin(1000, "uusd")]);
    deps.querier.with_token_balances(&[(
        TOKEN_CONTRACT,
        &[(MOCK_CONTRACT_ADDR, &Uint128::new(1000))],
    )]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        Denom::Native("uusd".to_string()),
        Some(Uint128::new(1001)),
    );
    expect_generic_err(&result, "Insufficient balance");

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        Denom::Token(TOKEN_CONTRACT.to_string()),
        Some(Uint128::new(1001)),
    );
    expect_generic_err(&result, "Insufficient balance");

    super::update_activation::will_success(&mut deps, true);

    super::participate::will_success(&mut deps, "Participator1", None);

    let result = exec(
        &mut deps,
        contract_env(),
        campaign_admin_sender(),
        Denom::Token(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        Some(Uint128::new(1000).checked_sub(CAMPAIGN_DISTRIBUTION_AMOUNTS[0]).unwrap() + Uint128::new(1)),
    );
    expect_generic_err(&result, "Insufficient balance");
}
