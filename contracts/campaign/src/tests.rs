use crate::entrypoints::{execute, instantiate, query};
use crate::mock_querier::mock_dependencies;
use crate::states::{BoosterState, CampaignState, ContractConfig, Participation};

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, StdError,
    Uint128, WasmMsg,
};

use cw20::Denom as Cw20Denom;
use valkyrie::campaign::{
    enumerations::{Denom, Referrer},
    execute_msgs::{DistributeResult, Distribution, ExecuteMsg, InstantiateMsg},
    query_msgs::{
        CampaignInfoResponse, CampaignStateResponse, DistributionConfigResponse, QueryMsg,
    },
};
use valkyrie::distributor::execute_msgs::ExecuteMsg as DistributorExecuteMsg;
use valkyrie::errors::ContractError;

const MOCK_CREATOR: &str = "creator";
const MOCK_GOVERNANCE: &str = "governance";
const MOCK_DISTRIBUTOR: &str = "distributor";
const MOCK_TOKEN_CONTRACT: &str = "valkyrie";
const MOCK_FACTORY: &str = "factory";
const MOCK_BURN_CONTRACT: &str = "burn";
const MOCK_TITLE: &str = "campaign_title";
const MOCK_URL: &str = "campaign_url";
const MOCK_DESCRIPTION: &str = "campaign_description";
const MOCK_PARAMETER_KEY: &str = "parameter_key";
const MOCK_DISTRIBUTION_DENOM: &str = "uusd";
const MOCK_DISTRIBUTION_AMOUNTS: &[u128; 3] = &[100000000u128, 80000000u128, 60000000u128];

fn init(deps: DepsMut) {
    let msg = InstantiateMsg {
        governance: MOCK_GOVERNANCE.to_string(),
        distributor: MOCK_DISTRIBUTOR.to_string(),
        token_contract: MOCK_TOKEN_CONTRACT.to_string(),
        factory: MOCK_FACTORY.to_string(),
        burn_contract: MOCK_BURN_CONTRACT.to_string(),
        title: MOCK_TITLE.to_string(),
        url: MOCK_URL.to_string(),
        description: MOCK_DESCRIPTION.to_string(),
        parameter_key: MOCK_PARAMETER_KEY.to_string(),
        distribution_denom: Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string()),
        distribution_amounts: MOCK_DISTRIBUTION_AMOUNTS
            .iter()
            .map(|v| Uint128(*v))
            .collect(),
    };

    let _res = instantiate(deps, mock_env(), mock_info(MOCK_CREATOR, &[]), msg).unwrap();
}

fn activate(deps: DepsMut) {
    let msg = ExecuteMsg::UpdateActivation { active: true };

    let _res = execute(deps, mock_env(), mock_info(MOCK_CREATOR, &[]), msg).unwrap();
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    init(deps.as_mut());

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::CampaignState {}).unwrap();
    let campaign_state: CampaignStateResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        CampaignStateResponse {
            participation_count: 0u64,
            cumulative_distribution_amount: Uint128::zero(),
            locked_balance: Uint128::zero(),
            balance: Uint128::zero(),
            is_active: false,
        },
        campaign_state,
    );

    let env = mock_env();
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::CampaignInfo {}).unwrap();
    let campaign_info: CampaignInfoResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        CampaignInfoResponse {
            title: MOCK_TITLE.to_string(),
            url: MOCK_URL.to_string(),
            description: MOCK_DESCRIPTION.to_string(),
            parameter_key: MOCK_PARAMETER_KEY.to_string(),
            creator: MOCK_CREATOR.to_string(),
            created_at: env.block.time,
        },
        campaign_info
    );

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::DistributionConfig {}).unwrap();
    let distribution_config: DistributionConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        DistributionConfigResponse {
            denom: Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string()),
            amounts: vec![
                Uint128::from(100000000u128),
                Uint128::from(80000000u128),
                Uint128::from(60000000u128)
            ],
        },
        distribution_config
    );
}

#[test]
fn update_campaign_info() {
    let mut deps = mock_dependencies(&[]);

    init(deps.as_mut());
    let info = mock_info(MOCK_CREATOR, &[]);
    let env = mock_env();

    let msg = ExecuteMsg::UpdateCampaignInfo {
        title: Some("campaign_title_2".to_string()),
        url: Some("campaign_url_2".to_string()),
        description: Some("campaign_description_2".to_string()),
    };

    let wrong_info = mock_info("addr0001", &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), wrong_info, msg.clone()),
        Err(ContractError::Unauthorized {})
    );
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::CampaignInfo {}).unwrap();
    let campaign_info: CampaignInfoResponse = from_binary(&query_res).unwrap();
    assert_eq!("campaign_title_2", campaign_info.title);
    assert_eq!("campaign_url_2".to_string(), campaign_info.url);
    assert_eq!("campaign_description_2", campaign_info.description);
    assert_eq!(MOCK_PARAMETER_KEY, campaign_info.parameter_key);
    assert_eq!(MOCK_CREATOR, campaign_info.creator);
    assert_eq!(env.block.time, campaign_info.created_at);
}

#[test]
fn update_distribution_config() {
    let mut deps = mock_dependencies(&[]);

    init(deps.as_mut());

    let info = mock_info(MOCK_CREATOR, &[]);
    let env = mock_env();

    let msg = ExecuteMsg::UpdateDistributionConfig {
        denom: Denom::Native("ukrw".to_string()),
        amounts: vec![
            Uint128::from(120000000u128),
            Uint128::from(100000000u128),
            Uint128::from(80000000u128),
        ],
    };

    let wrong_info = mock_info("addr0001", &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), wrong_info, msg.clone()),
        Err(ContractError::Unauthorized {})
    );

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::DistributionConfig {}).unwrap();
    let distribution_config: DistributionConfigResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        DistributionConfigResponse {
            denom: Denom::Native("ukrw".to_string()),
            amounts: vec![
                Uint128::from(120000000u128),
                Uint128::from(100000000u128),
                Uint128::from(80000000u128),
            ],
        },
        distribution_config
    );
}

#[test]
fn update_admin() {
    let mut deps = mock_dependencies(&[]);

    init(deps.as_mut());
    let info = mock_info(MOCK_CREATOR, &[]);
    let env = mock_env();

    let msg = ExecuteMsg::UpdateAdmin {
        address: "addr0000".to_string(),
    };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        ContractConfig::load(&deps.storage).unwrap().admin,
        Addr::unchecked("addr0000")
    );
}

#[test]
fn update_activation() {
    let mut deps = mock_dependencies(&[]);

    init(deps.as_mut());
    let info = mock_info(MOCK_CREATOR, &[]);
    let env = mock_env();

    let msg = ExecuteMsg::UpdateActivation { active: true };
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::CampaignState {}).unwrap();
    let campaign_state: CampaignStateResponse = from_binary(&query_res).unwrap();
    assert_eq!(true, campaign_state.is_active,);
}

#[test]
fn withdraw_reward_at_pending() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);

    init(deps.as_mut());

    deps.querier.with_tax(
        Decimal::percent(5u64),
        &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
    );

    deps.querier
        .with_valkyrie_config(100u64, Decimal::percent(10));

    // unauthorized
    let msg = ExecuteMsg::WithdrawReward {
        denom: Denom::Native("uusd".to_string()),
        amount: None,
    };
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), info, msg),
        Err(ContractError::Unauthorized {})
    );

    // Insufficient balance
    let msg = ExecuteMsg::WithdrawReward {
        denom: Denom::Native("uusd".to_string()),
        amount: Some(Uint128::from(10_000_000_001u128)),
    };
    let info = mock_info(MOCK_CREATOR, &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), info.clone(), msg),
        Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )))
    );

    // set locked balance
    let mut campaign_state: CampaignState = CampaignState::load(&deps.storage).unwrap();
    campaign_state.locked_balance.push((
        cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string()),
        Uint128::from(1000000000u128),
    ));
    campaign_state.save(&mut deps.storage).unwrap();

    // normal
    let msg = ExecuteMsg::WithdrawReward {
        denom: Denom::Native("uusd".to_string()),
        amount: None,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        vec![
            attr("action", "withdraw_reward"),
            attr("receive_amount", "9000000000uusd"),
            attr("burn_amount", "0uusd"),
        ],
        res.attributes,
    );

    assert_eq!(
        vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: MOCK_CREATOR.to_string(),
            amount: vec![Coin {
                amount: Uint128::from(8999000000u128),
                denom: "uusd".to_string()
            }],
        })],
        res.messages,
    )
}

#[test]
fn withdraw_reward_at_active() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);

    init(deps.as_mut());
    activate(deps.as_mut());

    deps.querier.with_tax(
        Decimal::percent(5u64),
        &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
    );

    deps.querier
        .with_valkyrie_config(100u64, Decimal::percent(10));

    // unauthorized
    let msg = ExecuteMsg::WithdrawReward {
        denom: Denom::Native("uusd".to_string()),
        amount: None,
    };
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), info, msg),
        Err(ContractError::Unauthorized {})
    );

    // Insufficient balance
    let msg = ExecuteMsg::WithdrawReward {
        denom: Denom::Native("uusd".to_string()),
        amount: Some(Uint128::from(10_000_000_001u128)),
    };
    let info = mock_info(MOCK_CREATOR, &[]);
    assert_eq!(
        execute(deps.as_mut(), env.clone(), info.clone(), msg),
        Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )))
    );

    // normal
    let msg = ExecuteMsg::WithdrawReward {
        denom: Denom::Native("uusd".to_string()),
        amount: None,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        vec![
            attr("action", "withdraw_reward"),
            attr("receive_amount", "9090909090uusd"),
            attr("burn_amount", "909090910uusd"),
        ],
        res.attributes,
    );

    assert_eq!(
        vec![
            CosmosMsg::Bank(BankMsg::Send {
                to_address: MOCK_BURN_CONTRACT.to_string(),
                amount: vec![Coin {
                    amount: Uint128::from(908090910u128),
                    denom: "uusd".to_string()
                }],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: MOCK_CREATOR.to_string(),
                amount: vec![Coin {
                    amount: Uint128::from(9089909090u128),
                    denom: "uusd".to_string()
                }],
            })
        ],
        res.messages,
    )
}

#[test]
fn claim_reward() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);
    let env = mock_env();

    init(deps.as_mut());
    activate(deps.as_mut());

    deps.querier.with_tax(
        Decimal::percent(5u64),
        &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
    );

    let denom = cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string());

    // set locked balance
    let mut campaign_state: CampaignState = CampaignState::load(&deps.storage).unwrap();
    campaign_state
        .locked_balance
        .push((denom.clone(), Uint128::from(1000000000u128)));
    campaign_state.save(&mut deps.storage).unwrap();

    let participation: Participation = Participation {
        actor_address: Addr::unchecked("addr0000"),
        referrer_address: None,
        rewards: vec![(denom, Uint128::from(10000000u128))],
        participated_at: env.block.time.clone(),
        booster_rewards: Uint128::from(1000000u128),
        drop_booster_claimable: false,
    };

    participation.save(&mut deps.storage).unwrap();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::ClaimReward {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        vec![
            attr("action", "claim_reward"),
            attr("booster_rewards", "1000000valkyrie"),
            attr("reward[uusd]", "10000000uusd"),
        ],
        res.attributes
    );

    assert_eq!(
        vec![
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "addr0000".to_string(),
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(9523809u128)
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_DISTRIBUTOR.to_string(),
                send: vec![],
                msg: to_binary(&DistributorExecuteMsg::Spend {
                    recipient: "addr0000".to_string(),
                    amount: Uint128::from(1000000u128),
                })
                .unwrap(),
            })
        ],
        res.messages,
    );

    assert_eq!(
        vec![(
            Cw20Denom::Native("uusd".to_string()),
            Uint128::from(990000000u128)
        )],
        CampaignState::load(&deps.storage).unwrap().locked_balance
    );
}

#[test]
fn claim_reward_with_drop_booster() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);
    let env = mock_env();

    init(deps.as_mut());
    activate(deps.as_mut());

    deps.querier.with_tax(
        Decimal::percent(5u64),
        &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
    );

    let denom = cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string());

    // set locked balance
    let mut campaign_state: CampaignState = CampaignState::load(&deps.storage).unwrap();
    campaign_state
        .locked_balance
        .push((denom.clone(), Uint128::from(1000000000u128)));
    campaign_state.save(&mut deps.storage).unwrap();

    let booster_state: BoosterState = BoosterState {
        drop_booster_amount: Uint128::from(100000000u128),
        drop_booster_left_amount: Uint128::from(100000000u128),
        drop_booster_participations: 100u64,
        activity_booster_amount: Uint128::zero(),
        activity_booster_left_amount: Uint128::zero(),
        plus_booster_amount: Uint128::zero(),
        plus_booster_left_amount: Uint128::zero(),
        boosted_at: env.block.time.clone().minus_seconds(60),
    };
    booster_state.save(&mut deps.storage).unwrap();

    let participation: Participation = Participation {
        actor_address: Addr::unchecked("addr0000"),
        referrer_address: None,
        rewards: vec![(denom, Uint128::from(10000000u128))],
        participated_at: env.block.time.clone(),
        booster_rewards: Uint128::from(1000000u128),
        drop_booster_claimable: true,
    };

    participation.save(&mut deps.storage).unwrap();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::ClaimReward {};
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        vec![
            attr("action", "claim_reward"),
            attr("booster_rewards", "2000000valkyrie"),
            attr("reward[uusd]", "10000000uusd"),
        ],
        res.attributes
    );

    assert_eq!(
        vec![
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "addr0000".to_string(),
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(9523809u128)
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_DISTRIBUTOR.to_string(),
                send: vec![],
                msg: to_binary(&DistributorExecuteMsg::Spend {
                    recipient: "addr0000".to_string(),
                    amount: Uint128::from(2000000u128),
                })
                .unwrap(),
            })
        ],
        res.messages,
    );

    assert_eq!(
        vec![(
            Cw20Denom::Native("uusd".to_string()),
            Uint128::from(990000000u128)
        )],
        CampaignState::load(&deps.storage).unwrap().locked_balance
    );

    assert_eq!(
        BoosterState::load(&deps.storage)
            .unwrap()
            .drop_booster_left_amount,
        Uint128::from(99000000u128),
    );
}

#[test]
fn participate() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);
    let env = mock_env();

    init(deps.as_mut());

    // deactivated campaign participation
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("addr0000", &[]),
            ExecuteMsg::Participate {
                referrer: Some(Referrer::Address("addr0001".to_string())),
            }
        ),
        Err(ContractError::Std(StdError::generic_err(
            "Deactivated campaign",
        )))
    );

    activate(deps.as_mut());
    let denom = cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string());
    let participation: Participation = Participation {
        actor_address: Addr::unchecked("addr0000"),
        referrer_address: None,
        rewards: vec![(denom, Uint128::from(10000000u128))],
        participated_at: env.block.time.clone(),
        booster_rewards: Uint128::from(1000000u128),
        drop_booster_claimable: false,
    };

    participation.save(&mut deps.storage).unwrap();
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("addr0000", &[]),
            ExecuteMsg::Participate {
                referrer: Some(Referrer::Address("addr0001".to_string())),
            }
        ),
        Err(ContractError::Std(StdError::generic_err(
            "Already participated",
        )))
    );

    // store voting power
    deps.querier
        .with_voting_powers(&[(&"addr0001".to_string(), &Decimal::percent(1))]);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("addr0001", &[]),
        ExecuteMsg::Participate {
            referrer: Some(Referrer::Address("addr0000".to_string())),
        },
    )
    .unwrap();

    assert_eq!(
        vec![
            attr("action", "participate"),
            attr("actor", "addr0001"),
            attr("activity_booster", "0valkyrie"),
            attr("plus_booster", "0valkyrie")
        ],
        res.attributes
    );
    assert_eq!(
        DistributeResult {
            actor_address: "addr0001".to_string(),
            reward_denom: Denom::Native("uusd".to_string()),
            configured_reward_amount: Uint128::from(240000000u128),
            distributed_reward_amount: Uint128::from(180000000u128),
            distributions: vec![
                Distribution {
                    address: "addr0001".to_string(),
                    distance: 0u64,
                    rewards: vec![(
                        Denom::Native("uusd".to_string()),
                        Uint128::from(100000000u128)
                    ),],
                },
                Distribution {
                    address: "addr0000".to_string(),
                    distance: 1u64,
                    rewards: vec![(
                        Denom::Native("uusd".to_string()),
                        Uint128::from(80000000u128)
                    )],
                },
            ]
        },
        from_binary::<DistributeResult>(&res.data.unwrap()).unwrap()
    );

    assert_eq!(
        CampaignState::load(&deps.storage)
            .unwrap()
            .participation_count,
        1u64
    );
    assert_eq!(
        true,
        Participation::load(&deps.storage, &Addr::unchecked("addr0001"))
            .unwrap()
            .drop_booster_claimable
    );
}

#[test]
fn participate_with_booster() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);
    let env = mock_env();

    init(deps.as_mut());

    // deactivated campaign participation
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("addr0000", &[]),
            ExecuteMsg::Participate {
                referrer: Some(Referrer::Address("addr0001".to_string())),
            }
        ),
        Err(ContractError::Std(StdError::generic_err(
            "Deactivated campaign",
        )))
    );

    activate(deps.as_mut());
    let denom = cw20::Denom::Native(MOCK_DISTRIBUTION_DENOM.to_string());
    let participation: Participation = Participation {
        actor_address: Addr::unchecked("addr0000"),
        referrer_address: None,
        rewards: vec![(denom, Uint128::from(10000000u128))],
        participated_at: env.block.time.clone(),
        booster_rewards: Uint128::from(1000000u128),
        drop_booster_claimable: false,
    };

    participation.save(&mut deps.storage).unwrap();
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("addr0000", &[]),
            ExecuteMsg::Participate {
                referrer: Some(Referrer::Address("addr0001".to_string())),
            }
        ),
        Err(ContractError::Std(StdError::generic_err(
            "Already participated",
        )))
    );

    // store booster state
    let booster_state: BoosterState = BoosterState {
        drop_booster_amount: Uint128::from(1000000000u128),
        drop_booster_left_amount: Uint128::from(1000000000u128),
        drop_booster_participations: 1000u64,
        activity_booster_amount: Uint128::from(800000000u128),
        activity_booster_left_amount: Uint128::from(800000000u128),
        plus_booster_amount: Uint128::from(100000000u128),
        plus_booster_left_amount: Uint128::from(100000000u128),
        boosted_at: env.block.time.clone().minus_seconds(60),
    };
    booster_state.save(&mut deps.storage).unwrap();

    // store voting power
    deps.querier
        .with_voting_powers(&[(&"addr0001".to_string(), &Decimal::percent(1))]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0001", &[]),
        ExecuteMsg::Participate {
            referrer: Some(Referrer::Address("addr0000".to_string())),
        },
    )
    .unwrap();

    assert_eq!(
        vec![
            attr("action", "participate"),
            attr("actor", "addr0001"),
            attr("activity_booster", "800000valkyrie"),
            attr("plus_booster", "1000000valkyrie")
        ],
        res.attributes
    );
    assert_eq!(
        DistributeResult {
            actor_address: "addr0001".to_string(),
            reward_denom: Denom::Native("uusd".to_string()),
            configured_reward_amount: Uint128::from(240000000u128),
            distributed_reward_amount: Uint128::from(180000000u128),
            distributions: vec![
                Distribution {
                    address: "addr0001".to_string(),
                    distance: 0u64,
                    rewards: vec![
                        (
                            Denom::Native("uusd".to_string()),
                            Uint128::from(100000000u128)
                        ),
                        (
                            Denom::Token("valkyrie".to_string()),
                            Uint128::from(1333333u128) // 1000000(plus) + 333333(activity)
                        ),
                    ],
                },
                Distribution {
                    address: "addr0000".to_string(),
                    distance: 1u64,
                    rewards: vec![
                        (
                            Denom::Native("uusd".to_string()),
                            Uint128::from(80000000u128)
                        ),
                        (
                            Denom::Token("valkyrie".to_string()),
                            Uint128::from(266666u128)
                        )
                    ],
                },
            ]
        },
        from_binary::<DistributeResult>(&res.data.unwrap()).unwrap()
    );

    assert_eq!(
        CampaignState::load(&deps.storage)
            .unwrap()
            .participation_count,
        1u64
    );

    assert_eq!(
        BoosterState::load(&deps.storage).unwrap(),
        BoosterState {
            drop_booster_amount: Uint128::from(1000000000u128),
            drop_booster_left_amount: Uint128::from(1000000000u128),
            drop_booster_participations: 1000u64,
            activity_booster_amount: Uint128::from(800000000u128),
            activity_booster_left_amount: Uint128::from(799200000u128),
            plus_booster_amount: Uint128::from(100000000u128),
            plus_booster_left_amount: Uint128::from(99000000u128),
            boosted_at: env.block.time.clone().minus_seconds(60),
        }
    );
}

#[test]
fn register_booster() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);
    let env = mock_env();

    init(deps.as_mut());

    let msg = ExecuteMsg::RegisterBooster {
        drop_booster_amount: Uint128::from(1000000u128),
        activity_booster_amount: Uint128::from(2000000u128),
        plus_booster_amount: Uint128::from(3000000u128),
    };
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("addr0000", &[]),
            msg.clone()
        ),
        Err(ContractError::Unauthorized {})
    );

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(MOCK_DISTRIBUTOR, &[]),
        msg,
    )
    .unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "register_booster"),
            attr("drop_booster_amount", Uint128::from(1000000u128)),
            attr("drop_booster_participations", "0"),
            attr("activity_booster_amount", Uint128::from(2000000u128)),
            attr("plus_booster_amount", Uint128::from(3000000u128)),
        ]
    );

    assert_eq!(
        BoosterState::load(&deps.storage).unwrap(),
        BoosterState {
            drop_booster_amount: Uint128::from(1000000u128),
            drop_booster_left_amount: Uint128::from(1000000u128),
            drop_booster_participations: 0u64,
            activity_booster_amount: Uint128::from(2000000u128),
            activity_booster_left_amount: Uint128::from(2000000u128),
            plus_booster_amount: Uint128::from(3000000u128),
            plus_booster_left_amount: Uint128::from(3000000u128),
            boosted_at: env.block.time,
        }
    )
}

#[test]
fn deregister_booster() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(10000000000u128),
    }]);

    init(deps.as_mut());

    let msg = ExecuteMsg::RegisterBooster {
        drop_booster_amount: Uint128::from(1000000u128),
        activity_booster_amount: Uint128::from(2000000u128),
        plus_booster_amount: Uint128::from(3000000u128),
    };

    let _res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_DISTRIBUTOR, &[]),
        msg,
    )
    .unwrap();

    let msg = ExecuteMsg::DeregisterBooster {};
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("addr0000", &[]),
            msg.clone()
        ),
        Err(ContractError::Unauthorized {})
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(MOCK_DISTRIBUTOR, &[]),
        msg,
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "deregister_booster"),
            attr("drop_booster_left_amount", Uint128::from(1000000u128)),
            attr("activity_booster_left_amount", Uint128::from(2000000u128)),
            attr("plus_booster_left_amount", Uint128::from(3000000u128)),
        ]
    );

    assert_eq!(BoosterState::load(&deps.storage).is_err(), true);
}
