use valkyrie::governance::messages::{InstantiateMsg, ExecuteMsg, Cw20HookMsg};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{attr, from_binary, to_binary, Addr, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, Storage, Api, Querier};
use cw20::Cw20ReceiveMsg;
use super::state::Config;
use super::errors::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    super::contracts::instantiate(deps, env, _info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateGovernanceConfig {
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        } => super::contracts::update_governance_config(
            deps,
            env,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        ),
        ExecuteMsg::SetBoostContract { .. } => {}
        ExecuteMsg::CastVote { .. } => {}
        ExecuteMsg::WithdrawVotingTokens { .. } => {}
        ExecuteMsg::EndPoll { .. } => {}
        ExecuteMsg::ExecutePoll { .. } => {}
        ExecuteMsg::ExpirePoll { .. } => {}
        ExecuteMsg::SnapshotPoll { .. } => {}
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> HandleResult {
    // only asset contract can execute this message
    let config: Config = config_read(&deps.storage).load()?;
    if config.valkyrie_token != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::unauthorized());
    }

    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::StakeVotingTokens {} => {
                stake_voting_tokens(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
            Cw20HookMsg::CreatePoll {
                title,
                description,
                link,
                execution,
            } => create_poll(
                deps,
                env,
                cw20_msg.sender,
                cw20_msg.amount,
                title,
                description,
                link,
                execution,
            ),
        }
    } else {
        Err(StdError::generic_err("data should be given"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {

}