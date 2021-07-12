use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, CosmosMsg, Uint128, coin, BankMsg};
use valkyrie::common::ContractResult;
use crate::executions::claim_participation_reward;
use valkyrie::test_utils::{contract_env, default_sender, expect_generic_err, DEFAULT_SENDER};
use cosmwasm_std::testing::mock_info;
use crate::tests::{CAMPAIGN_DISTRIBUTION_AMOUNTS, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE};
use crate::states::{Participation, CampaignState};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    claim_participation_reward(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, sender: &str) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    let participator = Addr::unchecked("Participator");
    super::participate::will_success(&mut deps, participator.as_str(), None);

    let (_, _, response) = will_success(&mut deps, participator.as_str());
    assert_eq!(response.messages, vec![
        CosmosMsg::Bank(BankMsg::Send {
            to_address: participator.to_string(),
            amount: vec![coin(
                CAMPAIGN_DISTRIBUTION_AMOUNTS[0].u128(),
                CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string(),
            )]
        }),
    ]);

    let participation = Participation::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation.reward_amount, Uint128::zero());

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.locked_balance, Uint128::zero());
}

#[test]
fn failed_no_reward() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, DEFAULT_SENDER, None);

    will_success(&mut deps, DEFAULT_SENDER);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
    );
    expect_generic_err(&result, "Not exist participation reward");
}
