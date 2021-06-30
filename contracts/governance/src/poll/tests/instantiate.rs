use cosmwasm_std::{Decimal, Env, MessageInfo, Response, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::governance::execute_msgs::PollConfigInitMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};

use crate::poll::executions::instantiate;
use crate::poll::states::{PollConfig, PollState};
use crate::tests::{default_env, default_info, expect_generic_err, POLL_EXECUTION_DELAY_PERIOD, POLL_PROPOSAL_DEPOSIT, POLL_QUORUM_PERCENT, POLL_SNAPSHOT_PERIOD, POLL_THRESHOLD_PERCENT, POLL_VOTING_PERIOD};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    quorum: Decimal,
    threshold: Decimal,
    voting_period: u64,
    execution_delay_period: u64,
    proposal_deposit: Uint128,
    snapshot_period: u64,
) -> ContractResult<Response> {
    let msg = PollConfigInitMsg {
        quorum,
        threshold,
        voting_period,
        execution_delay_period,
        proposal_deposit,
        snapshot_period,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = default_env();
    let info = default_info();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        Decimal::percent(POLL_QUORUM_PERCENT),
        Decimal::percent(POLL_THRESHOLD_PERCENT),
        POLL_VOTING_PERIOD,
        POLL_EXECUTION_DELAY_PERIOD,
        POLL_PROPOSAL_DEPOSIT,
        POLL_SNAPSHOT_PERIOD,
    ).unwrap();

    (env, info, response)
}


#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    default(&mut deps);

    let poll_config = PollConfig::load(&deps.storage).unwrap();
    assert_eq!(poll_config.quorum, Decimal::percent(POLL_QUORUM_PERCENT));
    assert_eq!(poll_config.threshold, Decimal::percent(POLL_THRESHOLD_PERCENT));
    assert_eq!(poll_config.voting_period, POLL_VOTING_PERIOD);
    assert_eq!(poll_config.execution_delay_period, POLL_EXECUTION_DELAY_PERIOD);
    assert_eq!(poll_config.proposal_deposit, POLL_PROPOSAL_DEPOSIT);
    assert_eq!(poll_config.snapshot_period, POLL_SNAPSHOT_PERIOD);

    let poll_state = PollState::load(&deps.storage).unwrap();
    assert_eq!(poll_state.poll_count, 0);
    assert_eq!(poll_state.total_deposit, Uint128::zero());
}

#[test]
fn failed_invalid_threshold() {
    let mut deps = custom_deps(&[]);

    let result = exec(
        &mut deps,
        default_env(),
        default_info(),
        Decimal::percent(101),
        Decimal::percent(POLL_THRESHOLD_PERCENT),
        POLL_VOTING_PERIOD,
        POLL_EXECUTION_DELAY_PERIOD,
        POLL_PROPOSAL_DEPOSIT,
        POLL_SNAPSHOT_PERIOD,
    );

    expect_generic_err(&result, "quorum must be 0 to 1");
}

#[test]
fn failed_invalid_quorum() {
    let mut deps = custom_deps(&[]);

    let result = exec(
        &mut deps,
        default_env(),
        default_info(),
        Decimal::percent(POLL_QUORUM_PERCENT),
        Decimal::percent(101),
        POLL_VOTING_PERIOD,
        POLL_EXECUTION_DELAY_PERIOD,
        POLL_PROPOSAL_DEPOSIT,
        POLL_SNAPSHOT_PERIOD,
    );

    expect_generic_err(&result, "threshold must be 0 to 1");
}