use cosmwasm_std::{Addr, Env, MessageInfo, Response, Uint128, SubMsg, CosmosMsg, WasmMsg, to_binary};
use cosmwasm_std::testing::mock_info;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::expect_generic_err;

use crate::executions::participate;
use crate::states::{CampaignState, Actor};
use valkyrie::test_constants::campaign::{campaign_env, PARTICIPATION_REWARD_AMOUNT, REFERRAL_REWARD_AMOUNTS, PARTICIPATION_REWARD_DENOM_NATIVE, COLLATERAL_AMOUNT};
use valkyrie::test_constants::{default_sender, DEFAULT_SENDER};
use valkyrie::test_constants::campaign_manager::REFERRAL_REWARD_TOKEN;
use valkyrie::campaign_manager::query_msgs::ReferralRewardLimitOptionResponse;
use cw20::{Denom, Cw20ExecuteMsg};
use valkyrie::governance::query_msgs::StakerStateResponse;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    actor: String,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    participate(deps.as_mut(), env, info, actor, referrer)
}

pub fn will_success(
    deps: &mut CustomDeps,
    participator: &str,
    referrer: Option<Referrer>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = mock_info(participator, &[]);

    super::deposit_collateral::will_success(deps, participator, COLLATERAL_AMOUNT);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        participator.to_string(),
        referrer,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed_without_referrer() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::deposit::will_success(
        &mut deps,
        PARTICIPATION_REWARD_AMOUNT.u128() * 2,
        1000000000,
    );

    let participator = Addr::unchecked("Participator");

    let (env, _, _) = will_success(&mut deps, participator.as_str(), None);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.actor_count, 1);
    assert_eq!(campaign_state.participation_count, 1);
    assert_eq!(campaign_state.last_active_height, Some(env.block.height));
    assert_eq!(campaign_state.locked_balances, vec![
        (cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()), PARTICIPATION_REWARD_AMOUNT),
    ]);

    let participation = Actor::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation, Actor {
        address: participator.clone(),
        referrer: None,
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        referral_reward_amount: Uint128::zero(),
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        cumulative_referral_reward_amount: Uint128::zero(),
        participation_count: 1,
        referral_count: 0,
        last_participated_at: env.block.time,
    });
}

#[test]
fn succeed_with_referrer() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::deposit::will_success(
        &mut deps,
        PARTICIPATION_REWARD_AMOUNT.u128() * 2,
        REFERRAL_REWARD_AMOUNTS[0].u128(),
    );

    let participator = Addr::unchecked("Participator");
    let referrer = Addr::unchecked("Referrer");

    let (referrer_env, _, _) = will_success(
        &mut deps,
        referrer.as_str(),
        Some(Referrer::Address("InvalidReferrer".to_string())),
    );

    let (env, _, _) = will_success(
        &mut deps,
        participator.as_str(),
        Some(Referrer::Address(referrer.to_string())),
    );

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.actor_count, 2);
    assert_eq!(campaign_state.participation_count, 2);
    assert_eq!(campaign_state.last_active_height, Some(env.block.height));
    assert_eq!(campaign_state.locked_balances, vec![
        (cw20::Denom::Native(PARTICIPATION_REWARD_DENOM_NATIVE.to_string()), PARTICIPATION_REWARD_AMOUNT.checked_mul(Uint128::new(2)).unwrap()),
        (cw20::Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN)), REFERRAL_REWARD_AMOUNTS[0]),
    ]);

    let participation = Actor::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation, Actor {
        address: participator.clone(),
        referrer: Some(referrer.clone()),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        referral_reward_amount: Uint128::zero(),
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        cumulative_referral_reward_amount: Uint128::zero(),
        participation_count: 1,
        referral_count: 0,
        last_participated_at: env.block.time,
    });

    let referrer_participation = Actor::load(&deps.storage, &referrer).unwrap();
    assert_eq!(referrer_participation, Actor {
        address: referrer.clone(),
        referrer: None,
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        referral_reward_amount: REFERRAL_REWARD_AMOUNTS[0],
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        cumulative_referral_reward_amount: REFERRAL_REWARD_AMOUNTS[0],
        participation_count: 1,
        referral_count: 1,
        last_participated_at: referrer_env.block.time,
    });
}

#[test]
fn succeed_twice() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::deposit::will_success(&mut deps, 1000, 10000000000000);

    let participator = Addr::unchecked("participator");

    will_success(&mut deps, participator.as_str(), None);
    let (env, _, _) = will_success(&mut deps, participator.as_str(), None);

    let participation = Actor::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation, Actor {
        address: participator.clone(),
        referrer: None,
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT.checked_mul(Uint128::new(2)).unwrap(),
        referral_reward_amount: Uint128::zero(),
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT.checked_mul(Uint128::new(2)).unwrap(),
        cumulative_referral_reward_amount: Uint128::zero(),
        participation_count: 2,
        referral_count: 0,
        last_participated_at: env.block.time,
    });

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.actor_count, 1);
    assert_eq!(campaign_state.participation_count, 2);
}

#[test]
fn failed_inactive_campaign() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_env(),
        default_sender(),
        DEFAULT_SENDER.to_string(),
        None,
    );

    expect_generic_err(&result, "Inactive campaign");
}

#[test]
fn failed_insufficient_balance() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::deposit::will_success(&mut deps, 5, 10000000000000);

    will_success(&mut deps, "Participator1", None);

    super::deposit_collateral::will_success(&mut deps, "Participator2", COLLATERAL_AMOUNT);

    let result = exec(
        &mut deps,
        campaign_env(),
        mock_info("Participator2", &[]),
        "Participator2".to_string(),
        None,
    );
    expect_generic_err(&result, "Insufficient balance");
}

#[test]
fn overflow_referral_reward() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::deposit::will_success(&mut deps, 100, 100);

    deps.querier.with_referral_reward_limit_option(ReferralRewardLimitOptionResponse {
        overflow_amount_recipient: None,
        base_count: 1,
        percent_for_governance_staking: 10,
    });

    deps.querier.with_gov_staker_state(
        "Referrer",
        StakerStateResponse {
            balance: Uint128::new(80),
            share: Uint128::new(80),
            votes: vec![],
        }
    );

    will_success(&mut deps, "Referrer", None);

    will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));
    will_success(&mut deps, "Participator2", Some(Referrer::Address("Participator".to_string())));
    will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));

    let referrer = Actor::load(&deps.storage, &Addr::unchecked("Referrer")).unwrap();
    assert_eq!(referrer.referral_reward_amount, Uint128::new(10)); //reach limit. overflow amount = 3

    let state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(state.balance(&Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))).available(), Uint128::new(85));



    deps.querier.with_gov_staker_state(
        "Referrer",
        StakerStateResponse {
            balance: Uint128::new(140),
            share: Uint128::new(140),
            votes: vec![],
        }
    );

    will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));

    let referrer = Actor::load(&deps.storage, &Addr::unchecked("Referrer")).unwrap();
    assert_eq!(referrer.referral_reward_amount, Uint128::new(14)); //reach limit. overflow amount = 1

    let state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(state.balance(&Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))).available(), Uint128::new(81));


    deps.querier.with_gov_staker_state(
        "Referrer",
        StakerStateResponse {
            balance: Uint128::new(160),
            share: Uint128::new(160),
            votes: vec![],
        }
    );

    deps.querier.with_referral_reward_limit_option(ReferralRewardLimitOptionResponse {
        overflow_amount_recipient: Some("Recipient".to_string()),
        base_count: 1,
        percent_for_governance_staking: 10,
    });

    let (_, _, response) = will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));

    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: REFERRAL_REWARD_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "Recipient".to_string(),
                amount: Uint128::new(3),
            }).unwrap(),
        })),
    ]);

    let referrer = Actor::load(&deps.storage, &Addr::unchecked("Referrer")).unwrap();
    assert_eq!(referrer.referral_reward_amount, Uint128::new(16)); //reach limit. overflow amount = 3

    let state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(state.balance(&Denom::Cw20(Addr::unchecked(REFERRAL_REWARD_TOKEN))).available(), Uint128::new(76));
}
