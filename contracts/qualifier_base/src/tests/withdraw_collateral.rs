use crate::tests::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, Addr, SubMsg, CosmosMsg, BankMsg, coin, StdError};
use crate::executions::{ExecuteResult, withdraw_collateral};
use cosmwasm_std::testing::{mock_env, mock_info};
use crate::states::Collateral;
use crate::tests::{COLLATERAL_LOCK_PERIOD, COLLATERAL_AMOUNT};
use crate::errors::ContractError;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ExecuteResult {
    withdraw_collateral(deps.as_mut(), env, info, amount)
}

pub fn will_success(
    deps: &mut CustomDeps,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info("Actor", &[]);

    let response = exec(deps, env.clone(), info.clone(), amount).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    let (env,_ , _) = super::deposit_collateral::will_success(&mut deps, "Actor", Uint128::new(1000));

    let mut collateral = Collateral::load(&deps.storage, &Addr::unchecked("Actor")).unwrap();
    collateral.locked_amounts = vec![(COLLATERAL_AMOUNT, env.block.height + COLLATERAL_LOCK_PERIOD)];
    collateral.save(&mut deps.storage).unwrap();

    let withdraw_amount = Uint128::new(1000).checked_sub(COLLATERAL_AMOUNT).unwrap();
    let (_, _, response) = will_success(&mut deps, withdraw_amount);
    assert_eq!(response.messages, vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: "Actor".to_string(),
        amount: vec![coin(withdraw_amount.u128(), "uusd")],
    }))]);
}

#[test]
fn failed_overdrawn() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    let (env, _, _) = super::deposit_collateral::will_success(&mut deps, "Actor", Uint128::new(1000));

    let mut collateral = Collateral::load(&deps.storage, &Addr::unchecked("Actor")).unwrap();
    collateral.locked_amounts = vec![(COLLATERAL_AMOUNT, env.block.height + COLLATERAL_LOCK_PERIOD)];
    collateral.save(&mut deps.storage).unwrap();

    let withdraw_amount = Uint128::new(1000).checked_sub(COLLATERAL_AMOUNT).unwrap() + Uint128::new(1);
    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("Actor", &[]),
        withdraw_amount,
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Overdraw collateral")));
}
