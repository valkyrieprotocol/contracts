use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Decimal, Uint128, Response};
use valkyrie::common::ContractResult;
use crate::poll::executions::update_poll_config;
use crate::tests::{init_default, POLL_QUORUM_PERCENT, POLL_THRESHOLD_PERCENT, POLL_VOTING_PERIOD, POLL_EXECUTION_DELAY_PERIOD, POLL_PROPOSAL_DEPOSIT, POLL_SNAPSHOT_PERIOD};
use crate::poll::states::PollConfig;
use cosmwasm_std::testing::{mock_info, MOCK_CONTRACT_ADDR};
use valkyrie::test_utils::{contract_env, expect_generic_err, default_sender, expect_unauthorized_err};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    execution_delay_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> ContractResult<Response> {
    update_poll_config(
        deps.as_mut(),
        env,
        info,
        quorum,
        threshold,
        voting_period,
        execution_delay_period,
        proposal_deposit,
        snapshot_period,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    execution_delay_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        quorum,
        threshold,
        voting_period,
        execution_delay_period,
        proposal_deposit,
        snapshot_period,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let quorum = Decimal::percent(POLL_QUORUM_PERCENT / 2);
    let threshold = Decimal::percent(POLL_THRESHOLD_PERCENT / 2);
    let voting_period = POLL_VOTING_PERIOD + 100;
    let execution_delay_period = POLL_EXECUTION_DELAY_PERIOD + 100;
    let proposal_deposit = POLL_PROPOSAL_DEPOSIT + Uint128(100);
    let snapshot_period = POLL_SNAPSHOT_PERIOD + 100;

    will_success(
        &mut deps,
        Some(quorum),
        Some(threshold),
        Some(voting_period),
        Some(execution_delay_period),
        Some(proposal_deposit),
        Some(snapshot_period),
    );

    let config = PollConfig::load(&deps.storage).unwrap();
    assert_eq!(config.quorum, quorum);
    assert_ne!(config.quorum, Decimal::percent(POLL_QUORUM_PERCENT));
    assert_eq!(config.threshold, threshold);
    assert_ne!(config.threshold, Decimal::percent(POLL_THRESHOLD_PERCENT));
    assert_eq!(config.voting_period, voting_period);
    assert_ne!(config.voting_period, POLL_VOTING_PERIOD);
    assert_eq!(config.execution_delay_period, execution_delay_period);
    assert_ne!(config.execution_delay_period, POLL_EXECUTION_DELAY_PERIOD);
    assert_eq!(config.proposal_deposit, proposal_deposit);
    assert_ne!(config.proposal_deposit, POLL_PROPOSAL_DEPOSIT);
    assert_eq!(config.snapshot_period, snapshot_period);
    assert_ne!(config.snapshot_period, POLL_SNAPSHOT_PERIOD);
}

#[test]
fn failed_invalid_threshold() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        Some(Decimal::percent(101)),
        Some(Decimal::percent(POLL_THRESHOLD_PERCENT)),
        Some(POLL_VOTING_PERIOD),
        Some(POLL_EXECUTION_DELAY_PERIOD),
        Some(POLL_PROPOSAL_DEPOSIT),
        Some(POLL_SNAPSHOT_PERIOD),
    );

    expect_generic_err(&result, "quorum must be 0 to 1");
}

#[test]
fn failed_invalid_quorum() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        Some(Decimal::percent(POLL_QUORUM_PERCENT)),
        Some(Decimal::percent(101)),
        Some(POLL_VOTING_PERIOD),
        Some(POLL_EXECUTION_DELAY_PERIOD),
        Some(POLL_PROPOSAL_DEPOSIT),
        Some(POLL_SNAPSHOT_PERIOD),
    );

    expect_generic_err(&result, "threshold must be 0 to 1");
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    expect_unauthorized_err(&result);
}