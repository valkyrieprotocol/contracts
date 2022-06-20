use cosmwasm_std::{Addr, BankMsg, coin, CosmosMsg, Env, MessageInfo, Response, SubMsg, Uint128};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::{campaign_env_height, PARTICIPATION_REWARD_AMOUNT, PARTICIPATION_REWARD_DENOM_NATIVE, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1, PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3, PARTICIPATOR1};
use valkyrie::test_utils::expect_generic_err;

use crate::executions::claim_participation_reward;
use crate::states::{Actor, CampaignState};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    claim_participation_reward(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, height: u64, sender: &str) -> (Env, MessageInfo, Response) {
    let env = campaign_env_height(height);
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 3995, 3000);

    let participator = Addr::unchecked(PARTICIPATOR1);
    let (env, _, _) = super::participate::will_success(
        &mut deps,
        participator.as_str(),
        None,
    );

    let (_, _, response) = will_success(
        &mut deps,
        env.block.height + PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3.1,
        participator.as_str(),
    );

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: participator.to_string(),
            amount: vec![coin(
                3000, //subtracted tax
                PARTICIPATION_REWARD_DENOM_NATIVE.to_string(),
            )],
        })),
    ]);

    let participation = Actor::load(&deps.storage, &participator).unwrap();
    // assert_eq!(participation.participation_reward_amounts, vec![]);
    assert_eq!(participation.cumulative_participation_reward_amount, Uint128::new(3000));

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.locked_balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())),
        Uint128::zero(),
    );
    assert_eq!(
        campaign_state.balance(&cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string())).total,
        Uint128::new(995),
    );
}

#[test]
fn failed_no_reward() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, PARTICIPATION_REWARD_AMOUNT.u128(), 2000);

    let participator = Addr::unchecked(PARTICIPATOR1);
    let (env, _, _) = super::participate::will_success(
        &mut deps,
        &participator.as_str(),
        None,
    );

    let (_, info, _) = will_success(&mut deps, env.block.height + PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1.0, &participator.as_str());

    let result = exec(
        &mut deps,
        env,
        info,
    );

    expect_generic_err(&result, "Not exist claimable participation reward");
}
