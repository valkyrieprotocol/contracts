use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Response, to_binary, from_binary, Addr};

use valkyrie_qualifier::query_msgs::QueryMsg;

use crate::{executions, queries};
use crate::executions::ExecuteResult;
use crate::msgs::{ExecuteMsg, InstantiateMsg, Cw20HookMsg};
use crate::errors::ContractError;
use cw20::{Cw20ReceiveMsg, Denom};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    executions::instantiate(deps, env, info, msg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ExecuteResult {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateAdmin {
            address,
        } => executions::update_admin(deps, env, info, address),
        ExecuteMsg::UpdateRequirement {
            continue_option_on_fail,
            min_token_balances,
            min_luna_staking,
            collateral_amount,
            collateral_lock_period,
        } => executions::update_requirement(
            deps,
            env,
            info,
            continue_option_on_fail,
            min_token_balances,
            min_luna_staking,
            collateral_amount,
            collateral_lock_period,
        ),
        ExecuteMsg::DepositCollateral {} => {
            let sender = info.sender.clone();
            let funds = info.funds.iter()
                .map(|c| (Denom::Native(c.denom.clone()), c.amount))
                .collect();

            executions::deposit_collateral(deps, env, info, sender, funds)
        },
        ExecuteMsg::WithdrawCollateral {
            amount,
        } => executions::withdraw_collateral(deps, env, info, amount),
        ExecuteMsg::Qualify(msg) => executions::qualify(deps, env, info, msg),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ExecuteResult {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::DepositCollateral {} => {
            let sender = info.sender.clone();

            executions::deposit_collateral(
                deps,
                env,
                info,
                Addr::unchecked(cw20_msg.sender),
                vec![(Denom::Cw20(sender), cw20_msg.amount)],
            )
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let result = match msg {
        QueryMsg::Qualify(msg) => to_binary(&queries::qualify(deps, env, msg)?),
        QueryMsg::Requirement {} => to_binary(&queries::requirement(deps, env)?),
    }?;

    Ok(result)
}
