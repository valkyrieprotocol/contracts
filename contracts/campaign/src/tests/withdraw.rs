use cosmwasm_std::{Addr, BankMsg, coin, CosmosMsg, Env, MessageInfo, Response, StdError, SubMsg, Uint128};

use crate::states::Deposit;
use valkyrie::mock_querier::{CustomDeps, custom_deps};
use crate::executions::withdraw;
use valkyrie::test_constants::campaign::{campaign_env, DEPOSIT_AMOUNT, DEPOSIT_LOCK_PERIOD};
use cosmwasm_std::testing::mock_info;
use valkyrie::errors::ContractError;
use valkyrie::common::ContractResult;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    withdraw(deps.as_mut(), env, info, amount)
}

pub fn will_success(
    deps: &mut CustomDeps,
    sender: &str,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone(), amount).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    let (env,_ , _) = super::deposit::will_success(&mut deps, "Actor", Uint128::new(1000));

    let mut deposit = Deposit::load(&deps.storage, &Addr::unchecked("Actor")).unwrap();
    deposit.locked_amounts = vec![(DEPOSIT_AMOUNT, env.block.height + DEPOSIT_LOCK_PERIOD)];
    deposit.save(&mut deps.storage).unwrap();

    let withdraw_amount = Uint128::new(1000).checked_sub(DEPOSIT_AMOUNT).unwrap();
    let (_, _, response) = will_success(&mut deps, "Actor", withdraw_amount);
    assert_eq!(response.messages, vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: "Actor".to_string(),
        amount: vec![coin(withdraw_amount.u128(), "uluna")],
    }))]);
}

#[test]
fn failed_overdrawn() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    let (env, _, _) = super::deposit::will_success(&mut deps, "Actor", Uint128::new(1000));

    let mut deposit = Deposit::load(&deps.storage, &Addr::unchecked("Actor")).unwrap();
    deposit.locked_amounts = vec![(DEPOSIT_AMOUNT, env.block.height + DEPOSIT_LOCK_PERIOD)];
    deposit.save(&mut deps.storage).unwrap();

    let withdraw_amount = Uint128::new(1000).checked_sub(DEPOSIT_AMOUNT).unwrap() + Uint128::new(1);
    let result = exec(
        &mut deps,
        campaign_env(),
        mock_info("Actor", &[]),
        withdraw_amount,
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Overdraw deposit")));
}
