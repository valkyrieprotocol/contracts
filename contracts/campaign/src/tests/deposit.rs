use cosmwasm_std::{Addr, coin, Env, MessageInfo, Response, StdError, Uint128};

use crate::states::Deposit;
use valkyrie::mock_querier::{CustomDeps, custom_deps};
use crate::executions::deposit;
use valkyrie::common::ContractResult;
use valkyrie::test_constants::campaign::{campaign_env, DEPOSIT_DENOM_NATIVE};
use cosmwasm_std::testing::mock_info;
use valkyrie::errors::ContractError;
use cw20::Denom;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    sender: &str,
    funds: Vec<(Denom, Uint128)>,
) -> ContractResult<Response> {
    deposit(
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
    let env = campaign_env();
    let info = mock_info(sender, &[coin(amount.u128(), "uluna")]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        sender,
        vec![(Denom::Native(DEPOSIT_DENOM_NATIVE.to_string()), amount)],
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let actor = Addr::unchecked("Actor");

    let deposit = Deposit::load_or_new(&deps.storage, &actor).unwrap();
    assert_eq!(deposit, Deposit {
        owner: actor.clone(),
        deposit_amount: Uint128::zero(),
        locked_amounts: vec![],
    });

    will_success(&mut deps, actor.as_str(), Uint128::new(100));

    let deposit = Deposit::load_or_new(&deps.storage, &actor).unwrap();
    assert_eq!(deposit, Deposit {
        owner: actor.clone(),
        deposit_amount: Uint128::new(100),
        locked_amounts: vec![],
    });

    will_success(&mut deps, actor.as_str(), Uint128::new(100));

    let deposit = Deposit::load_or_new(&deps.storage, &actor).unwrap();
    assert_eq!(deposit, Deposit {
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
        campaign_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![],
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Missing deposit denom")));

    let result = exec(
        &mut deps,
        campaign_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![
            (Denom::Native("ukrw".to_string()), Uint128::new(100)),
        ],
    );
    assert_eq!(result.unwrap_err(), ContractError::Std(StdError::generic_err("Missing deposit denom")));

    let result = exec(
        &mut deps,
        campaign_env(),
        mock_info("Actor", &[]),
        "Actor",
        vec![
            (Denom::Native(DEPOSIT_DENOM_NATIVE.to_string()), Uint128::zero()),
        ],
    );
    assert_eq!(result.unwrap_err(), ContractError::InvalidZeroAmount {});
}
