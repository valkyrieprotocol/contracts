use cosmwasm_std::{DepsMut, Env, MessageInfo, Addr, Decimal, Uint128, Response, StdError};
use crate::entrypoints;
use valkyrie::governance::execute_msgs::{InstantiateMsg, PollConfigInitMsg, ContractConfigInitMsg, StakingConfigInitMsg};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

pub const CONTRACT_CREATOR: &str = "ContractCreator";
pub const DEFAULT_SENDER: &str = "DefaultSender";

// common config
pub const TOKEN_CONTRACT: &str = "TokenContractAddress";

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
    let env = default_env();
    let info = mock_info(CONTRACT_CREATOR, &[]);

    let msg = InstantiateMsg {
        contract_config: ContractConfigInitMsg {
            token_contract: TOKEN_CONTRACT.to_string(),
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

pub fn default_env() -> Env {
    let mut env = mock_env();

    env.contract.address = Addr::unchecked(MOCK_CONTRACT_ADDR);

    env
}

pub fn env_plus_height(env: &mut Env, amount: i64) {
    let amount_abs = amount.unsigned_abs();

    if amount.is_positive() {
        env.block.height += amount_abs;
        env.block.time = env.block.time.plus_seconds(amount_abs * 7);
    } else {
        env.block.height -= amount_abs;
        env.block.time = env.block.time.minus_seconds(amount_abs * 7);
    }
}

pub fn env_set_height(env: &mut Env, height: u64) {
    env_plus_height(env, (height - env.block.height) as i64)
}

pub fn default_info() -> MessageInfo {
    mock_info(DEFAULT_SENDER, &[])
}

pub fn expect_generic_err(result: &ContractResult<Response>, expect_msg: &str) {
    match result {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => assert_eq!(msg, expect_msg),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

pub fn expect_unauthorized_err(result: &ContractResult<Response>) {
    match result {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => {
            // do nothing
        },
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}