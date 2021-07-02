use cosmwasm_std::{Addr, coin, Decimal, Env, MessageInfo, Response, Uint128};
use cosmwasm_std::testing::mock_info;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{contract_env, default_sender, expect_generic_err};

use crate::executions::participate;
use crate::states::{CampaignState, Participation};
use crate::tests::{CAMPAIGN_DISTRIBUTION_AMOUNTS, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE};
use crate::tests::register_booster::{DROP_BOOSTER_AMOUNT, PLUS_BOOSTER_AMOUNT};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    participate(deps.as_mut(), env, info, referrer)
}

pub fn will_success(
    deps: &mut CustomDeps,
    participator: &str,
    referrer: Option<Referrer>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(participator, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        referrer,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed_without_referrer() {
    let mut deps = custom_deps(&[
        coin(100000000000u128, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    let participator = Addr::unchecked("Participator");

    let (env, _, _) = will_success(&mut deps, participator.as_str(), None);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.participation_count, 1);
    assert_eq!(campaign_state.last_active_block, Some(env.block.height));
    assert_eq!(campaign_state.locked_balance, vec![(
        cw20::Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        CAMPAIGN_DISTRIBUTION_AMOUNTS[0],
    )]);

    let participation = Participation::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation, Participation {
        actor_address: participator.clone(),
        referrer_address: None,
        rewards: vec![(
            cw20::Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
            CAMPAIGN_DISTRIBUTION_AMOUNTS[0],
        )],
        booster_rewards: Uint128::zero(),
        drop_booster_claimable: true,
        participated_at: env.block.time,
    });
}

#[test]
fn succeed_with_referrer() {
    let mut deps = custom_deps(&[
        coin(100000000000u128, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

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
    assert_eq!(campaign_state.participation_count, 2);
    assert_eq!(campaign_state.last_active_block, Some(env.block.height));
    assert_eq!(campaign_state.locked_balance, vec![(
        cw20::Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        CAMPAIGN_DISTRIBUTION_AMOUNTS[0] + CAMPAIGN_DISTRIBUTION_AMOUNTS[0] + CAMPAIGN_DISTRIBUTION_AMOUNTS[1],
    )]);

    let participation = Participation::load(&deps.storage, &participator).unwrap();
    assert_eq!(participation, Participation {
        actor_address: participator.clone(),
        referrer_address: Some(referrer.clone()),
        rewards: vec![(
            cw20::Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
            CAMPAIGN_DISTRIBUTION_AMOUNTS[0],
        )],
        booster_rewards: Uint128::zero(),
        drop_booster_claimable: true,
        participated_at: env.block.time,
    });

    let referrer_participation = Participation::load(&deps.storage, &referrer).unwrap();
    assert_eq!(referrer_participation, Participation {
        actor_address: referrer.clone(),
        referrer_address: None,
        rewards: vec![(
            cw20::Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
            CAMPAIGN_DISTRIBUTION_AMOUNTS[0] + CAMPAIGN_DISTRIBUTION_AMOUNTS[1],
        )],
        booster_rewards: Uint128::zero(),
        drop_booster_claimable: true,
        participated_at: referrer_env.block.time,
    });
}

#[test]
fn succeed_with_booster() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);
    let activity_booster_multiplier = Decimal::percent(80);
    deps.querier.with_booster_config(
        Decimal::percent(10),
        Decimal::percent(80),
        Decimal::percent(10),
        activity_booster_multiplier.clone(),
    );

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    //minimum participation for boosting
    for i in 1..10 {
        will_success(&mut deps, format!("participator{}", i).as_str(), None);
    }

    let referrer = "Referrer";
    will_success(&mut deps,  referrer, None);

    super::register_booster::default(&mut deps);

    let participator = "Participator";
    will_success(
        &mut deps,
        participator,
        Some(Referrer::Address(referrer.to_string())),
    );

    let drop_booster = DROP_BOOSTER_AMOUNT.checked_div(Uint128(10)).unwrap();
    let activity_booster = activity_booster_multiplier * drop_booster;

    let referrer_participation = Participation::load(
        &deps.storage,
        &Addr::unchecked(referrer),
    ).unwrap();
    let reward_rate = Decimal::from_ratio(
        CAMPAIGN_DISTRIBUTION_AMOUNTS[1],
        CAMPAIGN_DISTRIBUTION_AMOUNTS.iter().sum::<Uint128>(),
    );
    assert_eq!(referrer_participation.booster_rewards, reward_rate * activity_booster);

    let participation = Participation::load(
        &deps.storage,
        &Addr::unchecked(participator),
    ).unwrap();
    let reward_rate = Decimal::from_ratio(
        CAMPAIGN_DISTRIBUTION_AMOUNTS[0],
        CAMPAIGN_DISTRIBUTION_AMOUNTS.iter().sum::<Uint128>(),
    );
    assert_eq!(participation.booster_rewards, reward_rate * activity_booster);

    let participator = "StakingParticipator";
    let voting_power = Decimal::percent(1);
    deps.querier.with_voting_powers(&[
        (&participator.to_string(), &voting_power),
    ]);

    will_success(&mut deps, participator, None);

    let participation = Participation::load(
        &deps.storage,
        &Addr::unchecked(participator),
    ).unwrap();
    let reward_rate = Decimal::from_ratio(
        CAMPAIGN_DISTRIBUTION_AMOUNTS[0],
        CAMPAIGN_DISTRIBUTION_AMOUNTS.iter().sum::<Uint128>(),
    );
    let plus_booster_amount = voting_power * PLUS_BOOSTER_AMOUNT;
    assert_eq!(participation.booster_rewards, (reward_rate * activity_booster) + plus_booster_amount);
}

#[test]
fn failed_inactive_campaign() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        None,
    );

    expect_generic_err(&result, "Inactive campaign");
}

#[test]
fn failed_twice() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    let (_, info, _) = will_success(&mut deps, "Participator1", None);

    let result = exec(&mut deps, contract_env(), info, None);

    expect_generic_err(&result, "Already participated");
}
