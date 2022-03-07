use cosmwasm_std::{Addr, Decimal, Uint128};

use valkyrie::mock_querier::{custom_deps};
use valkyrie::test_constants::campaign::{DEPOSIT_AMOUNT};

use crate::queries::{get_actor};

#[test]
fn calc_unlocked_reward() {
    let mut deps = custom_deps();
    deps.querier.with_tax(Decimal::percent(10), &[("uusd", &Uint128::new(100))]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 100000, 100000);

    let participator = Addr::unchecked("Participator");
    let (mut env, info, _) = super::participate::will_success(
        &mut deps,
        participator.as_str(),
        None,
    );

    assert_eq!(env.block.height, 12_345);

    // 0 blocks passed
    // happened anything.
    env.block.height = 12345 + 0;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::zero());

    // 9 blocks passed
    // happened anything.
    env.block.height = 12345 + 9;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::zero());

    // 10 blocks passed
    // participate #1 schedule1 distributed.
    // and started linear vesting distribution.
    env.block.height = 12345 + 10;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::new(300));

    // 14 blocks passed
    // progressing participate #1 linear vesting distribution.
    env.block.height = 12345 + 14;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::new(353));

    // 15 blocks passed, participated at 12360
    // progressing participate #1 linear vesting distribution.
    env.block.height = 12345 + 15;
    let participator = Addr::unchecked("Participator");
    super::deposit::will_success(&mut deps, participator.as_str(), DEPOSIT_AMOUNT);

    super::participate::exec(
        &mut deps,
        env.clone(),
        info.clone(),
        participator.to_string(),
        None,
    ).unwrap();

    // 25 blocks passed
    // progressing participate #1 linear vesting distribution.
    // participate #2 schedule1 distributed.
    env.block.height = 12345 + 25;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::new(799));

    // 201 blocks passed, participate #1 distributed completely.
    // ended participate #1 reward distribution.
    // progressing participate #2 linear vesting distribution.
    env.block.height = 12345 + 201;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::new(5788));

    // 215 blocks passed, app participate distributed completely.
    // ended participate #1 reward distribution.
    // ended participate #2 reward distribution.
    env.block.height = 12345 + 15 + 200;
    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation.unlocked_participation_reward_amount, Uint128::new(6000));
}