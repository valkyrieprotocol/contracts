use cosmwasm_std::{Addr, Env, MessageInfo, Response, Uint128, SubMsg, CosmosMsg, WasmMsg, to_binary, Reply, SubMsgExecutionResponse, Binary, from_binary};
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::ContractResult as ReplyResult;

use valkyrie::campaign::enumerations::Referrer;
use valkyrie::common::ContractResult;

use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::{expect_generic_err};

use crate::executions::{participate, participate_qualify_result, REPLY_QUALIFY_PARTICIPATION};
use crate::states::{CampaignState, Actor, CampaignConfig};
use valkyrie::test_constants::campaign::{campaign_env, PARTICIPATION_REWARD_AMOUNT, REFERRAL_REWARD_AMOUNTS, PARTICIPATION_REWARD_DENOM_NATIVE, DEPOSIT_AMOUNT, REFERRAL_REWARD_LOCK_PERIOD};
use valkyrie::test_constants::{default_sender, DEFAULT_SENDER, VALKYRIE_TOKEN};
use valkyrie::campaign_manager::query_msgs::ReferralRewardLimitOptionResponse;
use cw20::{Denom, Cw20ExecuteMsg};
use valkyrie::governance::query_msgs::StakerStateResponse;
use valkyrie_qualifier::QualificationResult;
use crate::proto::MsgExecuteContractResponse;
use protobuf::{Message};
use valkyrie::campaign::query_msgs::ActorResponse;
use crate::queries::get_actor;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    actor: String,
    referrer: Option<Referrer>,
) -> ContractResult<Response> {
    let response = participate(deps.as_mut(), env.clone(), info, actor.clone(), referrer);

    let campaign_config = CampaignConfig::load(deps.as_ref().storage).unwrap();

    if response.is_err() {
        return response;
    }

    let result:ContractResult<Response>;
    if campaign_config.qualifier == None {
        result = response;
    } else {
        //if has qualifier, call reply
        let mut msg = MsgExecuteContractResponse::default();
        msg.data = to_binary(&QualificationResult::success()).unwrap().to_vec();

        let qualify_response = participate_qualify_result(deps.as_mut(), env, Reply {
            id: REPLY_QUALIFY_PARTICIPATION,
            result: ReplyResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: Some(Binary::from(msg.write_to_bytes().unwrap())),
            })
        });

        if qualify_response.is_err() {
            return qualify_response;
        }

        result = qualify_response;
    }

    process_cw20(deps, &result);

    result
}

fn process_cw20(
    deps: &mut CustomDeps,
    response: &ContractResult<Response>
) {
    let messages = &response.as_ref().unwrap().messages;

    for message in messages {
        match &message.msg {
            CosmosMsg::Wasm(WasmMsg::Execute {contract_addr, funds: _, msg}) => {
                match from_binary(&msg).unwrap() {
                    Cw20ExecuteMsg::BurnFrom {owner, amount} => {
                        deps.querier.minus_token_balances(&[(contract_addr.as_str(), &[(&owner.as_str(), &amount)])]);
                    }
                    _ => {}
                }
            },
            _ => {}
        };
    };
}

pub fn will_success(
    deps: &mut CustomDeps,
    participator: &str,
    referrer: Option<Referrer>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = mock_info(participator, &[]);

    super::deposit::will_success(deps, participator, DEPOSIT_AMOUNT);

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
    super::add_reward_pool::will_success(
        &mut deps,
        PARTICIPATION_REWARD_AMOUNT.u128(),
        1000,
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

    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();
    assert_eq!(participation, ActorResponse {
        address: participator.to_string(),
        referrer_address: None,

        unlocked_participation_reward_amount: Uint128::zero(),
        claimed_participation_reward_amount: Uint128::zero(),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_amounts: vec![
            // (PARTICIPATION_REWARD_AMOUNT, env.block.height + PARTICIPATION_REWARD_LOCK_PERIOD),
            (PARTICIPATION_REWARD_AMOUNT, env.block.height + 0),
        ],
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT ,
        participation_reward_last_distributed: 0,

        unlocked_referral_reward_amount: Uint128::zero(),
        claimed_referral_reward_amount: Uint128::zero(),
        referral_reward_amount: Uint128::zero(),
        referral_reward_amounts: vec![],
        cumulative_referral_reward_amount: Uint128::zero(),
        referral_reward_last_distributed: 0,

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
    super::add_reward_pool::will_success(
        &mut deps,
        PARTICIPATION_REWARD_AMOUNT.u128() * 2,
        2000,
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
        (cw20::Denom::Cw20(Addr::unchecked(VALKYRIE_TOKEN)), REFERRAL_REWARD_AMOUNTS[0]),
    ]);

    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();

    assert_eq!(participation, ActorResponse {
        address: participator.to_string(),
        referrer_address: Some(referrer.to_string()),

        unlocked_participation_reward_amount: Uint128::zero(),
        claimed_participation_reward_amount: Uint128::zero(),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_amounts: vec![
            // (PARTICIPATION_REWARD_AMOUNT, env.block.height + PARTICIPATION_REWARD_LOCK_PERIOD),
            (PARTICIPATION_REWARD_AMOUNT, env.block.height + 0),
        ],
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT ,
        participation_reward_last_distributed: 0,

        unlocked_referral_reward_amount: Uint128::zero(),
        claimed_referral_reward_amount: Uint128::zero(),
        referral_reward_amount: Uint128::zero(),
        referral_reward_amounts: vec![],
        cumulative_referral_reward_amount: Uint128::zero(),
        referral_reward_last_distributed: 0,

        participation_count: 1,
        referral_count: 0,
        last_participated_at: env.block.time,
    });

    let referrer_participation = get_actor(deps.as_ref(), env.clone(), referrer.to_string()).unwrap();

    assert_eq!(referrer_participation, ActorResponse {
        address: referrer.to_string(),
        referrer_address: None,

        unlocked_participation_reward_amount: Uint128::zero(),
        claimed_participation_reward_amount: Uint128::zero(),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_amounts: vec![
            // (PARTICIPATION_REWARD_AMOUNT, env.block.height + PARTICIPATION_REWARD_LOCK_PERIOD),
            (PARTICIPATION_REWARD_AMOUNT, env.block.height + 0),
        ],
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT ,
        participation_reward_last_distributed: 0,

        unlocked_referral_reward_amount: Uint128::zero(),
        claimed_referral_reward_amount: Uint128::zero(),
        referral_reward_amount: REFERRAL_REWARD_AMOUNTS[0],
        referral_reward_amounts: vec![
            (REFERRAL_REWARD_AMOUNTS[0], env.block.height + REFERRAL_REWARD_LOCK_PERIOD)
        ],
        cumulative_referral_reward_amount: REFERRAL_REWARD_AMOUNTS[0],
        referral_reward_last_distributed: 0,

        participation_count: 1,
        referral_count: 1,
        last_participated_at: env.block.time,
    });
}

#[test]
fn succeed_twice() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 6000, 10000000000000);

    let participator = Addr::unchecked("participator");

    will_success(&mut deps, participator.as_str(), None);
    let (env, _, _) = will_success(&mut deps, participator.as_str(), None);

    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();

    assert_eq!(participation, ActorResponse {
        address: participator.to_string(),
        referrer_address: None,

        unlocked_participation_reward_amount: Uint128::zero(),
        claimed_participation_reward_amount: Uint128::zero(),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT + PARTICIPATION_REWARD_AMOUNT,
        participation_reward_amounts: vec![
            // (PARTICIPATION_REWARD_AMOUNT, env.block.height + PARTICIPATION_REWARD_LOCK_PERIOD),
            (PARTICIPATION_REWARD_AMOUNT, env.block.height + 0),
            (PARTICIPATION_REWARD_AMOUNT, env.block.height + 0),
        ],
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT + PARTICIPATION_REWARD_AMOUNT,
        participation_reward_last_distributed: 0,

        unlocked_referral_reward_amount: Uint128::zero(),
        claimed_referral_reward_amount: Uint128::zero(),
        referral_reward_amount: Uint128::zero(),
        referral_reward_amounts: vec![],
        cumulative_referral_reward_amount: Uint128::zero(),
        referral_reward_last_distributed: 0,

        participation_count: 2,
        referral_count: 0,
        last_participated_at: env.block.time,
    });

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.actor_count, 1);
    assert_eq!(campaign_state.participation_count, 2);
}

#[test]
fn succeed_with_burn_vp() {
    let mut deps = custom_deps();

    let vp_amount = Uint128::new(100);

    super::instantiate::default(&mut deps);
    super::update_campaign_config::will_success(
        &mut deps,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(vp_amount.clone()),
        None,
        None,
        None
    );
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, PARTICIPATION_REWARD_AMOUNT.u128(), 10000000000000);


    let config = CampaignConfig::load(deps.as_ref().storage).unwrap();
    let participator = Addr::unchecked("participator");

    deps.querier.plus_token_balances(&[(config.vp_token.as_str(), &[(&participator.as_str(), &vp_amount)])]);

    let (env, _, _) = will_success(&mut deps, participator.as_str(), None);

    let participation = get_actor(deps.as_ref(), env.clone(), participator.to_string()).unwrap();

    assert_eq!(participation, ActorResponse {
        address: participator.to_string(),
        referrer_address: None,

        unlocked_participation_reward_amount: Uint128::zero(),
        claimed_participation_reward_amount: Uint128::zero(),
        participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_amounts: vec![
            // (PARTICIPATION_REWARD_AMOUNT, env.block.height + PARTICIPATION_REWARD_LOCK_PERIOD),
            (PARTICIPATION_REWARD_AMOUNT, env.block.height + 0),
        ],
        cumulative_participation_reward_amount: PARTICIPATION_REWARD_AMOUNT,
        participation_reward_last_distributed: 0,

        unlocked_referral_reward_amount: Uint128::zero(),
        claimed_referral_reward_amount: Uint128::zero(),
        referral_reward_amount: Uint128::zero(),
        referral_reward_amounts: vec![],
        cumulative_referral_reward_amount: Uint128::zero(),
        referral_reward_last_distributed: 0,

        participation_count: 1,
        referral_count: 0,
        last_participated_at: env.block.time,
    });

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state.actor_count, 1);
    assert_eq!(campaign_state.participation_count, 1);
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
    super::add_reward_pool::will_success(&mut deps, PARTICIPATION_REWARD_AMOUNT.u128(), 10000000000000);

    will_success(&mut deps, "Participator1", None);

    super::deposit::will_success(&mut deps, "Participator2", DEPOSIT_AMOUNT);

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
    super::add_reward_pool::will_success(&mut deps, 100000, 100000);

    deps.querier.with_referral_reward_limit_option(ReferralRewardLimitOptionResponse {
        overflow_amount_recipient: None,
        base_count: 1,
        percent_for_governance_staking: 10,
    });

    deps.querier.with_gov_staker_state(
        "Referrer",
        StakerStateResponse {
            balance: Uint128::new(20),
            share: Uint128::new(20),
            votes: vec![],
        }
    );

    will_success(&mut deps, "Referrer", None);

    let (referrer_env, _, _) = will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));
    will_success(&mut deps, "Participator2", Some(Referrer::Address("Participator".to_string())));
    will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));

    //reward limit : 902
    let referrer = Actor::load(&deps.storage, &Addr::unchecked("Referrer")).unwrap();
    assert_eq!(referrer.referral_reward_amounts, vec![
        (Uint128::new(400), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
        (Uint128::new(300), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
        (Uint128::new(202), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
    ]); //reach limit. overflow amount = 1

    let state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(state.balance(&Denom::Cw20(Addr::unchecked(VALKYRIE_TOKEN))).available(), Uint128::new(100000-400-300-202-400));



    deps.querier.with_gov_staker_state(
        "Referrer",
        StakerStateResponse {
            balance: Uint128::new(40),
            share: Uint128::new(40),
            votes: vec![],
        }
    );

    let (referrer_env, _, _) = will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));

    //reward limit : 904
    let referrer = Actor::load(&deps.storage, &Addr::unchecked("Referrer")).unwrap();
    assert_eq!(referrer.referral_reward_amounts, vec![
        (Uint128::new(400), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
        (Uint128::new(300), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
        (Uint128::new(202), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
        (Uint128::new(2), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD),
    ]); //reach limit. overflow amount = 3

    let state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(state.balance(&Denom::Cw20(Addr::unchecked(VALKYRIE_TOKEN))).available(), Uint128::new(100000-400-300-202-400-2));


    deps.querier.with_gov_staker_state(
        "Referrer",
        StakerStateResponse {
            balance: Uint128::new(60),
            share: Uint128::new(60),
            votes: vec![],
        }
    );
    //reward limit : 906
    deps.querier.with_referral_reward_limit_option(ReferralRewardLimitOptionResponse {
        overflow_amount_recipient: Some("Recipient".to_string()),
        base_count: 1,
        percent_for_governance_staking: 10,
    });

    crate::tests::claim_referral_reward::will_success(
        &mut deps,
        referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD,
        "Referrer",
    );

    let (referrer_env, _, response) = will_success(&mut deps, "Participator", Some(Referrer::Address("Referrer".to_string())));

    assert_eq!(response.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "Recipient".to_string(),
                amount: Uint128::new(398),
            }).unwrap(),
        })),
    );

    let referrer = Actor::load(&deps.storage, &Addr::unchecked("Referrer")).unwrap();
    assert_eq!(referrer.referral_reward_amounts, vec![(Uint128::new(2), referrer_env.block.height + REFERRAL_REWARD_LOCK_PERIOD)]); //reach limit. overflow amount = 3

    let state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(state.balance(&Denom::Cw20(Addr::unchecked(VALKYRIE_TOKEN))).available(), Uint128::new(98296));
}