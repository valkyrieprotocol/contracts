use cosmwasm_std::{Addr, DepsMut, Env, from_binary, MessageInfo, Response, StdResult};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw20::Cw20ReceiveMsg;

use valkyrie::governance::messages::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};

use super::errors::ContractError;
use super::state::Config;
use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use crate::common::state::ContractConfig;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    crate::common::contracts::instantiate(&deps, &env, &info, msg.contract_config)?;
    crate::staking::contracts::instantiate(&deps, &env, &info)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateContractConfig {
            admin,
            boost_contract,
        } => crate::common::contracts::update_config(
            deps,
            env,
            info,
            admin,
            boost_contract,
        ),
        ExecuteMsg::UnstakeVotingToken {
            amount,
        } => crate::staking::contracts::unstake_voting_token(
            deps,
            env,
            info,
            amount,
        )
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    // only asset contract can execute this message
    let config = ContractConfig::singleton_read(deps.storage).load()?;
    if config.is_token_contract(deps.api.addr_canonicalize(info.sender.as_str())?) {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::StakeVotingToken {}) => crate::staking::contracts::stake_voting_token(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
        ),
        Err(err) => Err(ContractError::Std(err))
    }
}