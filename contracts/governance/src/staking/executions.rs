use cosmwasm_std::{Addr, attr, CanonicalAddr, DepsMut, Env, MessageInfo, Response, StdError, Storage, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::cw20::{create_send_msg_response, query_cw20_balance};
use valkyrie::errors::ContractError;
use valkyrie::governance::enumerations::PollStatus;

use crate::common::state::ContractConfig;
use crate::poll::state::{Poll, PollState};

use super::state::{StakerState, StakingState};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> ContractResult<Response> {
    let state = StakingState {
        total_share: Uint128::zero(),
    };

    state.save(deps.storage)?;

    Ok(Response::default())
}

pub fn stake_voting_token(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender: Addr,
    amount: Uint128,
) -> ContractResult<Response> {
    if amount.le(&Uint128::zero()) {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds sent")));
    }

    let contract_config = ContractConfig::load(deps.storage)?;
    let poll_state = PollState::load(deps.storage)?;
    let mut staking_state = StakingState::load(deps.storage)?;

    let sender_address = deps.api.addr_canonicalize(sender.as_str())?;
    let mut staker_state = StakerState::may_load(deps.storage, &sender_address)?
        .unwrap_or(StakerState::default(&sender_address));

    let contract_balance = query_cw20_balance(
        &deps.querier,
        deps.api,
        &contract_config.token_contract,
        &env.contract.address,
    )?;
    let total_balance = contract_balance.checked_sub(poll_state.total_deposit + amount)?;

    let share = if total_balance.is_zero() || staking_state.total_share.is_zero() {
        amount
    } else {
        amount.multiply_ratio(staking_state.total_share, total_balance)
    };

    staking_state.total_share += share;
    staker_state.save(deps.storage)?;

    staker_state.share += share;
    staker_state.save(deps.storage)?;

    Ok(
        Response {
            submessages: vec![],
            messages: vec![],
            attributes: vec![
                attr("action", "staking"),
                attr("sender", sender.as_str()),
                attr("share", share.to_string()),
                attr("amount", amount.to_string()),
            ],
            data: None,
        }
    )
}

// Withdraw amount if not staked. By default all funds will be withdrawn.
pub fn unstake_voting_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let sender_address = deps.api.addr_canonicalize(info.sender.as_str())?;
    let staker_state = StakerState::may_load(deps.storage, &sender_address)?;

    if let Some(mut staker_state) = staker_state {
        let contract_config = ContractConfig::load(deps.storage)?;
        let poll_state = PollState::load(deps.storage)?;
        let mut staking_state = StakingState::load(deps.storage)?;

        // Load total share & total balance except proposal deposit amount
        let total_share = staking_state.total_share.u128();
        let contract_balance = query_cw20_balance(
            &deps.querier,
            deps.api,
            &contract_config.token_contract,
            &env.contract.address,
        )?;
        let total_balance = contract_balance.checked_sub(poll_state.total_deposit)?.u128();

        let locked_balance = compute_locked_balance(deps.storage, &mut staker_state, &sender_address)?;
        let locked_share = locked_balance * total_share / total_balance;
        let user_share = staker_state.share.u128();

        let withdraw_share = amount
            .map(|v| std::cmp::max(v.multiply_ratio(total_share, total_balance).u128(), 1u128))
            .unwrap_or_else(|| user_share - locked_share);
        let withdraw_amount = amount
            .map(|v| v.u128())
            .unwrap_or_else(|| withdraw_share * total_balance / total_share);

        if locked_share + withdraw_share > user_share {
            Err(ContractError::Std(StdError::generic_err(
                "User is trying to withdraw too many tokens.",
            )))
        } else {
            let share = user_share - withdraw_share;
            staker_state.share = Uint128::from(share);
            staker_state.save(deps.storage)?;

            staking_state.total_share = Uint128::from(total_share - withdraw_share);
            staking_state.save(deps.storage)?;

            Ok(
                create_send_msg_response(
                    &deps.api.addr_humanize(&contract_config.token_contract)?,
                    &deps.api.addr_humanize(&sender_address)?,
                    withdraw_amount,
                    "withdraw",
                )
            )
        }
    } else {
        Err(ContractError::Std(StdError::generic_err("Nothing staked")))
    }
}

// removes not in-progress poll voter info & unlock tokens
// and returns the largest locked amount in participated polls.
fn compute_locked_balance(
    storage: &mut dyn Storage,
    staker_state: &mut StakerState,
    voter: &CanonicalAddr,
) -> ContractResult<u128> {
    // filter out not in-progress polls
    staker_state.locked_balance.retain(|(poll_id, _)| {
        let poll = Poll::load(storage, poll_id).unwrap();

        if poll.status != PollStatus::InProgress {
            Poll::remove_voter(storage, poll_id, voter)
        }

        poll.status == PollStatus::InProgress
    });

    Ok(
        staker_state.locked_balance.iter()
            .map(|(_, v)| v.balance.u128())
            .max()
            .unwrap_or_default()
    )
}