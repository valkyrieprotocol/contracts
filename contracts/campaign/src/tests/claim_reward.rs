use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, CosmosMsg, Uint128, coin, BankMsg};
use valkyrie::common::ContractResult;
use crate::executions::claim_reward;
use valkyrie::test_utils::contract_env;
use cosmwasm_std::testing::mock_info;
use crate::tests::{CAMPAIGN_DISTRIBUTION_AMOUNTS, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE};
use crate::states::{Participation, CampaignState};

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    claim_reward(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, sender: &str) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed_without_booster() {
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

// #[test]
// fn succeed_with_booster() {
//     let mut deps = custom_deps(&[]);
//
//     super::instantiate::default(&mut deps);
//     super::update_activation::will_success(&mut deps, true);
//
//     //minimum participation for boosting
//     for i in 1..10 {
//         super::participate::will_success(
//             &mut deps,
//             format!("participator{}", i).as_str(),
//             None,
//         );
//     }
//
//     let participator = "Participator";
//     super::participate::will_success(&mut deps, participator, None);
//
//     super::register_booster::default(&mut deps);
//
//     will_success(&mut deps, )
// }

// #[test]
// fn cdlaim_reward() {
//     let mut deps = mock_dependencies(&[Coin {
//         denom: "uusd".to_string(),
//         amount: Uint128::from(10000000000u128),
//     }]);
//     let env = mock_env();
//
//     init(deps.as_mut());
//     activate(deps.as_mut());
//
//     deps.querier.with_tax(
//         Decimal::percent(5u64),
//         &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
//     );
//
//     let denom = cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string());
//
//     // set locked balance
//     let mut campaign_state: CampaignState = CampaignState::load(&deps.storage).unwrap();
//     campaign_state
//         .locked_balance
//         .push((denom.clone(), Uint128::from(1000000000u128)));
//     campaign_state.save(&mut deps.storage).unwrap();
//
//     let participation: Participation = Participation {
//         actor_address: Addr::unchecked("addr0000"),
//         referrer_address: None,
//         rewards: vec![(denom, Uint128::from(10000000u128))],
//         participated_at: env.block.time.clone(),
//         booster_rewards: Uint128::from(1000000u128),
//         drop_booster_claimable: false,
//     };
//
//     participation.save(&mut deps.storage).unwrap();
//
//     let info = mock_info("addr0000", &[]);
//     let msg = ExecuteMsg::ClaimReward {};
//     let res = execute(deps.as_mut(), env, info, msg).unwrap();
//     assert_eq!(
//         vec![
//             attr("action", "claim_reward"),
//             attr("booster_rewards", "1000000valkyrie"),
//             attr("reward[uusd]", "10000000uusd"),
//         ],
//         res.attributes
//     );
//
//     assert_eq!(
//         vec![
//             CosmosMsg::Bank(BankMsg::Send {
//                 to_address: "addr0000".to_string(),
//                 amount: vec![Coin {
//                     denom: "uusd".to_string(),
//                     amount: Uint128::from(9523809u128)
//                 }],
//             }),
//             CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: MOCK_DISTRIBUTOR.to_string(),
//                 send: vec![],
//                 msg: to_binary(&DistributorExecuteMsg::Spend {
//                     recipient: "addr0000".to_string(),
//                     amount: Uint128::from(1000000u128),
//                 })
//                     .unwrap(),
//             })
//         ],
//         res.messages,
//     );
//
//     assert_eq!(
//         vec![(
//             Cw20Denom::Native("uusd".to_string()),
//             Uint128::from(990000000u128)
//         )],
//         CampaignState::load(&deps.storage).unwrap().locked_balance
//     );
// }
//
// #[test]
// fn claim_reward_with_drop_booster() {
//     let mut deps = mock_dependencies(&[Coin {
//         denom: "uusd".to_string(),
//         amount: Uint128::from(10000000000u128),
//     }]);
//     let env = mock_env();
//
//     init(deps.as_mut());
//     activate(deps.as_mut());
//
//     deps.querier.with_tax(
//         Decimal::percent(5u64),
//         &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
//     );
//
//     let denom = cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string());
//
//     // set locked balance
//     let mut campaign_state: CampaignState = CampaignState::load(&deps.storage).unwrap();
//     campaign_state
//         .locked_balance
//         .push((denom.clone(), Uint128::from(1000000000u128)));
//     campaign_state.save(&mut deps.storage).unwrap();
//
//     let booster_state: BoosterState = BoosterState {
//         drop_booster_amount: Uint128::from(100000000u128),
//         drop_booster_left_amount: Uint128::from(100000000u128),
//         drop_booster_participations: 100u64,
//         activity_booster_amount: Uint128::zero(),
//         activity_booster_left_amount: Uint128::zero(),
//         plus_booster_amount: Uint128::zero(),
//         plus_booster_left_amount: Uint128::zero(),
//         boosted_at: env.block.time.clone().minus_seconds(60),
//     };
//     booster_state.save(&mut deps.storage).unwrap();
//
//     let participation: Participation = Participation {
//         actor_address: Addr::unchecked("addr0000"),
//         referrer_address: None,
//         rewards: vec![(denom, Uint128::from(10000000u128))],
//         participated_at: env.block.time.clone(),
//         booster_rewards: Uint128::from(1000000u128),
//         drop_booster_claimable: true,
//     };
//
//     participation.save(&mut deps.storage).unwrap();
//
//     let info = mock_info("addr0000", &[]);
//     let msg = ExecuteMsg::ClaimReward {};
//     let res = execute(deps.as_mut(), env, info, msg).unwrap();
//     assert_eq!(
//         vec![
//             attr("action", "claim_reward"),
//             attr("booster_rewards", "2000000valkyrie"),
//             attr("reward[uusd]", "10000000uusd"),
//         ],
//         res.attributes
//     );
//
//     assert_eq!(
//         vec![
//             CosmosMsg::Bank(BankMsg::Send {
//                 to_address: "addr0000".to_string(),
//                 amount: vec![Coin {
//                     denom: "uusd".to_string(),
//                     amount: Uint128::from(9523809u128)
//                 }],
//             }),
//             CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: MOCK_DISTRIBUTOR.to_string(),
//                 send: vec![],
//                 msg: to_binary(&DistributorExecuteMsg::Spend {
//                     recipient: "addr0000".to_string(),
//                     amount: Uint128::from(2000000u128),
//                 })
//                     .unwrap(),
//             })
//         ],
//         res.messages,
//     );
//
//     assert_eq!(
//         vec![(
//             Cw20Denom::Native("uusd".to_string()),
//             Uint128::from(990000000u128)
//         )],
//         CampaignState::load(&deps.storage).unwrap().locked_balance
//     );
//
//     assert_eq!(
//         BoosterState::load(&deps.storage)
//             .unwrap()
//             .drop_booster_left_amount,
//         Uint128::from(99000000u128),
//     );
// }