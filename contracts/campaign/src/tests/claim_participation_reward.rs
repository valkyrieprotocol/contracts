use cosmwasm_std::{Addr, BankMsg, coin, CosmosMsg, Decimal, Env, MessageInfo, Response, SubMsg, Uint128};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{DEFAULT_SENDER, default_sender};
use valkyrie::test_constants::campaign::{campaign_env, PARTICIPATION_REWARD_DENOM_NATIVE};
use valkyrie::test_utils::expect_generic_err;

use crate::executions::claim_participation_reward;
use crate::states::{Actor, CampaignState};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    claim_participation_reward(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, sender: &str) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();
    deps.querier.with_tax(Decimal::percent(10), &[("uusd", &Uint128::new(100))]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 1000, 1000);

    let participator = Addr::unchecked("Participator");
    super::participate::will_success(&mut deps, participator.as_str(), None);

    let (_, _, response) = will_success(&mut deps, participator.as_str());
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: participator.to_string(),
            amount: vec![coin(
                4,
                PARTICIPATION_REWARD_DENOM_NATIVE.to_string(),
            )],
        })),
    ]);

    let participation = Actor::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation.participation_reward_amount, Uint128::zero());
    assert_eq!(participation.cumulative_participation_reward_amount, Uint128::new(5));

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.locked_balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())),
        Uint128::zero(),
    );
}

#[test]
fn failed_no_reward() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 1000, 1000);
    super::participate::will_success(&mut deps, DEFAULT_SENDER, None);

    will_success(&mut deps, DEFAULT_SENDER);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
    );
    expect_generic_err(&result, "Not exist participation reward");
}
