use cosmwasm_std::{Env, Addr, MessageInfo, Response, StdError};
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR, mock_info};
use crate::common::ContractResult;
use crate::errors::ContractError;

pub const DEFAULT_SENDER: &str = "DefaultSender";
pub const CONTRACT_CREATOR: &str = "ContractCreator";

const BLOCK_TIME_INTERVAL: u64 = 7;

pub fn contract_env() -> Env {
    let mut env = mock_env();

    env.contract.address = Addr::unchecked(MOCK_CONTRACT_ADDR);

    env
}

pub fn contract_env_height(height: u64) -> Env {
    let mut env = contract_env();

    set_height(&mut env, height);

    env
}

pub fn default_sender() -> MessageInfo {
    mock_info(DEFAULT_SENDER, &[])
}

pub fn contract_sender() -> MessageInfo {
    mock_info(MOCK_CONTRACT_ADDR, &[])
}

pub fn plus_height(env: &mut Env, amount: u64) {
    env.block.height += amount;
    env.block.time = env.block.time.plus_seconds(amount * BLOCK_TIME_INTERVAL);
}

pub fn minus_height(env: &mut Env, amount: u64) {
    env.block.height -= amount;
    env.block.time = env.block.time.minus_seconds(amount * BLOCK_TIME_INTERVAL);
}

pub fn set_height(env: &mut Env, height: u64) {
    let diff = height as i128 - env.block.height as i128;

    if diff.is_positive() {
        plus_height(env, diff as u64);
    } else {
        minus_height(env, diff as u64);
    }
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

pub fn expect_already_exists_err(result: &ContractResult<Response>) {
    match result {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::AlreadyExists {}) => {
            // do nothing
        },
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

pub fn expect_not_found_err(result: &ContractResult<Response>) {
    match result {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotFound {}) => {
            // do nothing
        },
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

pub fn expect_exceed_limit_err(result: &ContractResult<Response>) {
    match result {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::ExceedLimit {}) => {
            // do nothing
        },
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
