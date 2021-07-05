use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Uint128};
use cosmwasm_std::testing::{mock_env, mock_info};

use valkyrie::governance::execute_msgs::{ContractConfigInitMsg, InstantiateMsg, PollConfigInitMsg, StakingConfigInitMsg};
use valkyrie::test_utils::CONTRACT_CREATOR;

use crate::entrypoints;

// common config
pub const GOVERNANCE_TOKEN: &str = "TokenContractAddress";

// staking config
pub const WITHDRAW_DELAY: u64 = 94097;

// poll config
pub const POLL_QUORUM_PERCENT: u64 = 30;
pub const POLL_THRESHOLD_PERCENT: u64 = 50;
pub const POLL_VOTING_PERIOD: u64 = 10000u64;
pub const POLL_EXECUTION_DELAY_PERIOD: u64 = 10000u64;
pub const POLL_PROPOSAL_DEPOSIT: Uint128 = Uint128(10000000000u128);
pub const POLL_SNAPSHOT_PERIOD: u64 = 10u64;

pub fn init_default(deps: DepsMut) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(CONTRACT_CREATOR, &[]);

    let msg = InstantiateMsg {
        contract_config: ContractConfigInitMsg {
            governance_token: GOVERNANCE_TOKEN.to_string(),
        },
        staking_config: StakingConfigInitMsg {
            withdraw_delay: WITHDRAW_DELAY,
        },
        poll_config: PollConfigInitMsg {
            quorum: Decimal::percent(POLL_QUORUM_PERCENT),
            threshold: Decimal::percent(POLL_THRESHOLD_PERCENT),
            voting_period: POLL_VOTING_PERIOD,
            execution_delay_period: POLL_EXECUTION_DELAY_PERIOD,
            proposal_deposit: POLL_PROPOSAL_DEPOSIT,
            snapshot_period: POLL_SNAPSHOT_PERIOD,
        },
    };

    entrypoints::instantiate(deps, env.clone(), info.clone(), msg).unwrap();

    (env, info)
}