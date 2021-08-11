use crate::tests::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Addr, Response, coin, StdError};
use cw20::Denom;
use crate::executions::{ExecuteResult, deposit_collateral};
use cosmwasm_std::testing::{mock_env, mock_info};
use crate::tests::COLLATERAL_DENOM_NATIVE;
use crate::states::Collateral;
use crate::errors::ContractError;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    sender: &str,
    funds: Vec<(Denom, Uint128)>,
) -> ExecuteResult {
    deposit_collateral(
        deps.as_mut(),
        env,
        info,
        Addr::unchecked(sender),
        funds,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    sender: &str,
    amount: Uint128,
) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info(sender, &[coin(amount.u128(), "uusd")]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        sender,
        vec![(Denom::Native(COLLATERAL_DENOM_NATIVE.to_string()), amount)],
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let actor = Addr::unchecked("Actor");

    let collateral = Collateral::load_or_new(&deps.storage, &actor).unwrap();
    assert_eq!(collateral, Collateral {
        owner: actor.clone(),
        deposit_amount: Uint128::zero(),
        locked_amounts: vec![],
    });

    will_success(&mut deps, actor.as_str(), Uint128::new(100));

    let collateral = Collateral::load_or_new(&deps.storage, &actor).unwrap();
    assert_eq!(collateral, Collateral {
        owner: actor.clone(),
        deposit_amount: Uint128::new(100),
        locked_amounts: vec![],
    });

    will_success(&mut deps, actor.as_str(), Uint128::new(100));

    let collateral = Collateral::load_or_new(&deps.storage, &actor).unwrap();
    assert_eq!(collateral, Collateral {
        owner: actor.clone(),
        deposit_amount: Uint128::new(200),
        locked_amounts: vec![],
    });
}

#[test]
fn failed_invalid_funds() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![],
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Missing collateral denom")));

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![
            (Denom::Native("uluna".to_string()), Uint128::new(100)),
            (Denom::Native("ukrw".to_string()), Uint128::new(100)),
        ],
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Too many sent denom")));

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![
            (Denom::Native("ukrw".to_string()), Uint128::new(100)),
        ],
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Missing collateral denom")));

    let result = exec(
        &mut deps,
        mock_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![
            (Denom::Native(COLLATERAL_DENOM_NATIVE.to_string()), Uint128::zero()),
        ],
    );
    assert_eq!(result.unwrap_err(), ContractError::InvalidZeroAmount {});
}
