use cosmwasm_std::{Env, Response, MessageInfo, Uint128, coin, Addr, Decimal, SubMsg, CosmosMsg, WasmMsg, to_binary};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{CustomDeps, custom_deps};

use valkyrie::test_constants::campaign::{campaign_env, PARTICIPATION_REWARD_DENOM_NATIVE, CAMPAIGN_ADMIN};
use crate::states::CampaignState;
use crate::executions::{deposit, calc_deposit_fee_amount};
use cosmwasm_std::testing::mock_info;
use valkyrie::test_constants::campaign_manager::{REFERRAL_REWARD_TOKEN, KEY_DENOM_NATIVE, MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT};
use valkyrie::test_utils::expect_generic_err;
use cw20::Cw20ExecuteMsg;
use valkyrie::test_constants::fund_manager::FUND_MANAGER;
use valkyrie::fund_manager::execute_msgs::Cw20HookMsg;
use valkyrie::campaign_manager::query_msgs::ConfigResponse;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    participation_reward_amount: Uint128,
    referral_reward_amount: Uint128,
) -> ContractResult<Response> {
    deps.querier.with_terraswap_price(
        REFERRAL_REWARD_TOKEN.to_string(),
        KEY_DENOM_NATIVE.to_string(),
        1f64,
    );

    let contract_address = env.contract.address.to_string();
    let result = deposit(
        deps.as_mut(),
        env,
        info,
        participation_reward_amount,
        referral_reward_amount,
    );

    deps.querier.plus_native_balance(
        contract_address.as_str(),
        vec![coin(participation_reward_amount.u128(), PARTICIPATION_REWARD_DENOM_NATIVE)],
    );
    deps.querier.plus_token_balances(&[(REFERRAL_REWARD_TOKEN, &[
        (contract_address.as_str(), &referral_reward_amount),
    ])]);

    result
}

pub fn will_success(
    deps: &mut CustomDeps,
    participation_reward_amount: u128,
    referral_reward_amount: u128,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = mock_info(CAMPAIGN_ADMIN, &[
        coin(participation_reward_amount, PARTICIPATION_REWARD_DENOM_NATIVE),
    ]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        Uint128::new(participation_reward_amount),
        Uint128::new(referral_reward_amount),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let mut config = ConfigResponse::default();
    config.deposit_fee_rate = Decimal::percent(1);
    deps.querier.with_global_campaign_config(config);

    let (env, info, response) = will_success(
        &mut deps,
        100 - MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT as u128,
        MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT as u128,
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: Uint128::from(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: FUND_MANAGER.to_string(),
                amount: Uint128::new(1),
                msg: to_binary(&Cw20HookMsg::CampaignDepositFee {}).unwrap(),
            }).unwrap(),
        })),
    ]);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())).total,
        Uint128::from(100 - MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT),
    );
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))).total,
        Uint128::new(19),
    );
}

#[test]
fn succeed_more_referral_token() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let mut config = ConfigResponse::default();
    config.deposit_fee_rate = Decimal::percent(1);
    deps.querier.with_global_campaign_config(config);

    let (env, info, response) = will_success(
        &mut deps,
        100,
        200,
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: Uint128::new(200),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: FUND_MANAGER.to_string(),
                amount: Uint128::new(3),
                msg: to_binary(&Cw20HookMsg::CampaignDepositFee {}).unwrap(),
            }).unwrap(),
        })),
    ]);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())).total,
        Uint128::new(100),
    );
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))).total,
        Uint128::new(197),
    );
}

#[test]
fn succeed_zero_participation_token() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let mut config = ConfigResponse::default();
    config.deposit_fee_rate = Decimal::percent(1);
    deps.querier.with_global_campaign_config(config);

    let (env, info, response) = will_success(
        &mut deps,
        0,
        200,
    );
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: Uint128::new(200),
            }).unwrap(),
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: FUND_MANAGER.to_string(),
                amount: Uint128::new(2),
                msg: to_binary(&Cw20HookMsg::CampaignDepositFee {}).unwrap(),
            }).unwrap(),
        })),
    ]);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())).total,
        Uint128::zero(),
    );
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))).total,
        Uint128::new(198),
    );
}

#[test]
fn failed_zero_referral_token() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    let info = mock_info(CAMPAIGN_ADMIN, &[
        coin(1, PARTICIPATION_REWARD_DENOM_NATIVE),
    ]);

    let result = exec(
        &mut deps,
        campaign_env(),
        info,
        Uint128::new(1),
        Uint128::zero(),
    );
    expect_generic_err(&result, format!(
        "Referral reward rate must be greater than {}",
        Decimal::percent(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT).to_string(),
    ).as_str());
}

#[test]
fn failed_insufficient_referral_token() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let info = mock_info(CAMPAIGN_ADMIN, &[
        coin(100 - MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT as u128, PARTICIPATION_REWARD_DENOM_NATIVE),
    ]);

    let result = exec(
        &mut deps,
        campaign_env(),
        info,
        Uint128::from(100 - MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT),
        Uint128::from(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT - 1),
    );
    expect_generic_err(&result, format!(
        "Referral reward rate must be greater than {}",
        Decimal::percent(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT).to_string(),
    ).as_str());
}

#[test]
fn test_deposit_fee_amount() {
    let result = calc_deposit_fee_amount(
        Uint128::new(20),
        Decimal::percent(20),
        Decimal::percent(1),
    ).unwrap();
    assert_eq!(result, Uint128::new(1));

    let result = calc_deposit_fee_amount(
        Uint128::new(200),
        Decimal::percent(50),
        Decimal::percent(1),
    ).unwrap();
    assert_eq!(result, Uint128::new(4));

    let result = calc_deposit_fee_amount(
        Uint128::new(500),
        Decimal::percent(50),
        Decimal::percent(1),
    ).unwrap();
    assert_eq!(result, Uint128::new(10));

    let result = calc_deposit_fee_amount(
        Uint128::new(500),
        Decimal::percent(40),
        Decimal::percent(1),
    ).unwrap();
    assert_eq!(result, Uint128::new(12));

    let result = calc_deposit_fee_amount(
        Uint128::new(20),
        Decimal::percent(20),
        Decimal::percent(5),
    ).unwrap();
    assert_eq!(result, Uint128::new(5));

    let result = calc_deposit_fee_amount(
        Uint128::new(200),
        Decimal::percent(50),
        Decimal::percent(5),
    ).unwrap();
    assert_eq!(result, Uint128::new(20));

    let result = calc_deposit_fee_amount(
        Uint128::new(500),
        Decimal::percent(50),
        Decimal::percent(5),
    ).unwrap();
    assert_eq!(result, Uint128::new(50));

    let result = calc_deposit_fee_amount(
        Uint128::new(500),
        Decimal::percent(40),
        Decimal::percent(5),
    ).unwrap();
    assert_eq!(result, Uint128::new(62));
}
