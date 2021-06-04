use cosmwasm_std::{Addr, DepsMut, Env, from_binary, MessageInfo, Response};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw20::Cw20ReceiveMsg;

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::governance::messages::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};

use crate::common::state::ContractConfig;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let mut deps_mut = deps;

    crate::common::contracts::instantiate(deps_mut.branch(), env.clone(), info.clone(), msg.contract_config)?;
    crate::staking::contracts::instantiate(deps_mut.branch(), env.clone(), info.clone())?;
    crate::poll::contracts::instantiate(deps_mut.branch(), env.clone(), info.clone(), msg.poll_config)?;

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
        ),
        ExecuteMsg::UpdatePollConfig {
            quorum,
            threshold,
            voting_period,
            execution_delay_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        } => crate::poll::contracts::update_poll_config(
            deps,
            env,
            info,
            quorum,
            threshold,
            voting_period,
            execution_delay_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        ),
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => crate::poll::contracts::cast_vote(
            deps,
            env,
            info,
            poll_id,
            vote,
            amount,
        ),
        ExecuteMsg::EndPoll {
            poll_id,
        } => crate::poll::contracts::end_poll(
            deps,
            env,
            info,
            poll_id,
        ),
        ExecuteMsg::ExecutePoll {
            poll_id,
        } => crate::poll::contracts::execute_poll(
            deps,
            env,
            info,
            poll_id,
        ),
        ExecuteMsg::ExpirePoll {
            poll_id,
        } => crate::poll::contracts::expire_poll(
            deps,
            env,
            info,
            poll_id,
        ),
        ExecuteMsg::SnapshotPoll {
            poll_id,
        } => crate::poll::contracts::snapshot_poll(
            deps,
            env,
            info,
            poll_id,
        ),
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
        Ok(Cw20HookMsg::CreatePoll {
               title,
               description,
               link,
               execution,
           }) => crate::poll::contracts::create_poll(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
            title,
            description,
            link,
            execution,
        ),
        Err(err) => Err(ContractError::Std(err)),
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
//
// }