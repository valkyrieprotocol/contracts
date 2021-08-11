use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128, StdError, to_binary, Addr, CosmosMsg, BankMsg, coin, WasmMsg};
use valkyrie_qualifier::{QualificationMsg, QualifiedContinueOption, QualificationResult};
use cw20::{Denom, Cw20ExecuteMsg};
use crate::msgs::InstantiateMsg;
use crate::states::{Requirement, QualifierConfig, is_admin, Collateral, Querier};
use crate::errors::ContractError;


pub type ExecuteResult = Result<Response, ContractError>;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    let mut response = Response::new();
    response.add_attribute("action", "instantiate");

    QualifierConfig {
        admin: info.sender,
        continue_option_on_fail: msg.continue_option_on_fail,
    }.save(deps.storage)?;

    let collateral_denom = msg.collateral.as_ref().map(|c| c.0.clone());

    if let Some(Denom::Cw20(token)) = collateral_denom.as_ref() {
        deps.api.addr_validate(token.as_str())?;
    }

    Requirement {
        min_token_balances: msg.min_token_balances,
        min_luna_staking: msg.min_luna_staking,
        collateral_denom,
        collateral_amount: msg.collateral.map_or(Uint128::zero(), |c| c.1),
        collateral_lock_period: msg.collateral_lock_period.unwrap_or_default(),
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: String,
) -> ExecuteResult {
    if !is_admin(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new();
    response.add_attribute("action", "update_admin");

    let mut config = QualifierConfig::load(deps.storage)?;

    config.admin = deps.api.addr_validate(new_admin.as_str())?;

    config.save(deps.storage)?;

    Ok(response)
}

pub fn update_requirement(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    continue_option_on_fail: Option<QualifiedContinueOption>,
    min_token_balances: Option<Vec<(Denom, Uint128)>>,
    min_luna_staking: Option<Uint128>,
    collateral_amount: Option<Uint128>,
    collateral_lock_period: Option<u64>,
) -> ExecuteResult {
    if !is_admin(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new();
    response.add_attribute("action", "update_requirement");

    if let Some(continue_option_on_fail) = continue_option_on_fail {
        let mut config = QualifierConfig::load(deps.storage)?;

        config.continue_option_on_fail = continue_option_on_fail;
        response.add_attribute("is_updated_continue_option_on_fail", "true");

        config.save(deps.storage)?;
    }

    let mut requirement = Requirement::load(deps.storage)?;

    if let Some(min_token_balances) = min_token_balances {
        let is_valid = min_token_balances.iter().all(|(denom, min_balance)| {
            let valid_denom = match denom {
                Denom::Native(_) => true,
                Denom::Cw20(address) => deps.api.addr_validate(address.as_str()).is_ok(),
            };

            valid_denom && !min_balance.is_zero()
        });

        if !is_valid {
            return Err(ContractError::Std(StdError::generic_err("Invalid input min_token_balances")));
        }

        requirement.min_token_balances = min_token_balances;
        response.add_attribute("is_updated_min_token_balances", "true");
    }

    if let Some(min_luna_staking) = min_luna_staking {
        if min_luna_staking.is_zero() {
            return Err(ContractError::Std(StdError::generic_err("Invalid input min_luna_staking")));
        }

        requirement.min_luna_staking = min_luna_staking;
        response.add_attribute("is_updated_min_luna_staking", "true");
    }

    if let Some(collateral_amount) = collateral_amount {
        if collateral_amount.is_zero() {
            return Err(ContractError::Std(StdError::generic_err("Invalid input collateral_amount")));
        }

        requirement.collateral_amount = collateral_amount;
        response.add_attribute("is_updated_collateral_amount", "true");
    }

    if let Some(collateral_lock_period) = collateral_lock_period {
        requirement.collateral_lock_period = collateral_lock_period;
        response.add_attribute("is_updated_collateral_lock_period", "true");
    }

    requirement.save(deps.storage)?;

    Ok(response)
}

pub fn deposit_collateral(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    sender: Addr,
    funds: Vec<(Denom, Uint128)>,
) -> ExecuteResult {
    if funds.len() < 1 {
        return Err(ContractError::Std(StdError::generic_err("Missing collateral denom")));
    } else if funds.len() > 1 {
        return Err(ContractError::Std(StdError::generic_err("Too many sent denom")));
    }

    let (send_denom, send_amount) = &funds[0];

    if send_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut response = Response::new();
    response.add_attribute("action", "deposit_collateral");

    let requirement = Requirement::load(deps.storage)?;

    if let Some(collateral_denom) = requirement.collateral_denom {
        if *send_denom != collateral_denom {
            return Err(ContractError::Std(StdError::generic_err("Missing collateral denom")));
        }
    }

    let mut collateral = Collateral::load_or_new(deps.storage, &sender)?;

    collateral.deposit_amount += send_amount;

    collateral.save(deps.storage)?;

    response.add_attribute("deposit", send_amount.to_string());
    response.add_attribute("balance", collateral.deposit_amount.to_string());

    Ok(response)
}

pub fn qualify(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: QualificationMsg,
) -> ExecuteResult {
    let mut response = Response::new();

    response.add_attribute("action", "qualify");

    let actor = deps.api.addr_validate(msg.actor.as_str())?;

    let requirement = Requirement::load(deps.storage)?;
    let querier = Querier::new(&deps.querier);

    let mut collateral: Option<Collateral> = None;

    let collateral_balance = if requirement.require_collateral() {
        collateral = Some(Collateral::load_or_new(deps.storage, &actor)?);

        collateral.as_ref().unwrap().balance(env.block.height)?
    } else {
        Uint128::zero()
    };

    let (is_valid, error_msg) = requirement.is_satisfy_requirements(&querier, &actor, collateral_balance)?;
    let result = if is_valid {
        QualificationResult {
            continue_option: QualifiedContinueOption::Eligible,
            reason: None,
        }
    } else {
        let config = QualifierConfig::load(deps.storage)?;

        QualificationResult {
            continue_option: config.continue_option_on_fail,
            reason: Some(error_msg),
        }
    };

    if is_valid && requirement.require_collateral() {
        let collateral = collateral.as_mut().unwrap();

        collateral.lock(requirement.collateral_amount, env.block.height, requirement.collateral_lock_period)?;
        collateral.save(deps.storage)?;
    }

    response.add_attribute("qualified_continue_option", result.continue_option.to_string());

    response.set_data(to_binary(&result)?);

    Ok(response)
}

pub fn withdraw_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ExecuteResult {
    let mut response = Response::new();
    response.add_attribute("action", "withdraw_collateral");

    let mut collateral = Collateral::load(deps.storage, &info.sender)?;

    response.add_attribute("deposit_amount", collateral.deposit_amount.to_string());
    response.add_attribute("locked_amount", collateral.locked_amount(env.block.height));

    collateral.clear(env.block.height);

    let balance = collateral.balance(env.block.height)?;

    if balance.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    if balance < amount {
        return Err(ContractError::Std(StdError::generic_err("Overdraw collateral")));
    }

    collateral.deposit_amount = collateral.deposit_amount.checked_sub(amount)?;

    collateral.save(deps.storage)?;

    let requirement = Requirement::load(deps.storage)?;

    let send_message = match requirement.collateral_denom {
        Some(Denom::Native(denom)) => CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(amount.u128(), denom)],
        }),
        Some(Denom::Cw20(token)) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
        }),
        None => return Err(ContractError::Std(StdError::generic_err("No collateral"))),
    };
    response.add_message(send_message);

    Ok(response)
}
