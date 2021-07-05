#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError,
};
use cw20::Cw20ReceiveMsg;

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;
use valkyrie::governance::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};
use valkyrie::governance::query_msgs::QueryMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    crate::common::executions::instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        msg.contract_config,
    )?;
    crate::staking::executions::instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        msg.staking_config,
    )?;
    crate::poll::executions::instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        msg.poll_config,
    )?;

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
        ExecuteMsg::UpdateStakingConfig {
            withdraw_delay,
        } => crate::staking::executions::update_config(deps, env, info, withdraw_delay),
        ExecuteMsg::UnstakeGovernanceToken {
            amount,
        } => crate::staking::executions::unstake_governance_token(deps, env, info, amount),
        ExecuteMsg::WithdrawGovernanceToken {} => {
            crate::staking::executions::withdraw_governance_token(deps, env, info)
        },
        ExecuteMsg::UpdatePollConfig {
            quorum,
            threshold,
            voting_period,
            execution_delay_period,
            proposal_deposit,
            snapshot_period,
        } => crate::poll::executions::update_poll_config(
            deps,
            env,
            info,
            quorum,
            threshold,
            voting_period,
            execution_delay_period,
            proposal_deposit,
            snapshot_period,
        ),
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => crate::poll::executions::cast_vote(deps, env, info, poll_id, vote, amount),
        ExecuteMsg::SnapshotPoll {
            poll_id,
        } => crate::poll::executions::snapshot_poll(deps, env, info, poll_id),
        ExecuteMsg::EndPoll {
            poll_id,
        } => crate::poll::executions::end_poll(deps, env, info, poll_id),
        ExecuteMsg::ExecutePoll {
            poll_id,
        } => crate::poll::executions::execute_poll(deps, env, info, poll_id),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::StakeGovernanceToken {} => crate::staking::executions::stake_governance_token(
            deps,
            env,
            info,
            Addr::unchecked(cw20_msg.sender),
            cw20_msg.amount,
        ),
        Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execution,
        } => crate::poll::executions::create_poll(
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        crate::poll::executions::REPLY_EXECUTION => {
            crate::poll::executions::reply_execution(deps, env, msg)
        }
        _ => Err(ContractError::Std(StdError::not_found("reply_id"))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::ContractConfig {} => {
            to_binary(&crate::common::queries::get_contract_config(deps, env)?)
        }
        QueryMsg::StakingConfig {} => {
            to_binary(&crate::staking::queries::get_staking_config(deps, env)?)
        }
        QueryMsg::PollConfig {} => to_binary(&crate::poll::queries::get_poll_config(deps, env)?),
        QueryMsg::PollState {} => to_binary(&crate::poll::queries::get_poll_state(deps, env)?),
        QueryMsg::Poll { poll_id } => {
            to_binary(&crate::poll::queries::get_poll(deps, env, poll_id)?)
        }
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => to_binary(&crate::poll::queries::query_polls(
            deps,
            env,
            filter,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => to_binary(&crate::poll::queries::query_voters(
            deps,
            env,
            poll_id,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::StakingState {} => {
            to_binary(&crate::staking::queries::get_staking_state(deps, env)?)
        }
        QueryMsg::StakerState { address } => to_binary(&crate::staking::queries::get_staker_state(
            deps, env, address,
        )?),
        QueryMsg::VotingPower { address } => to_binary(&crate::staking::queries::get_voting_power(
            deps, env, address,
        )?),
        QueryMsg::Unstaking { address } => to_binary(
            &crate::staking::queries::get_unstaking(deps, env, address)?
        )
    }?;

    Ok(result)
}
