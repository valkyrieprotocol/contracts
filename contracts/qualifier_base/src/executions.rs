use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128, StdError, to_binary};
use valkyrie_qualifier::{QualificationMsg, QualifiedContinueOption, QualificationResult};
use cw20::Denom;
use crate::msgs::InstantiateMsg;
use crate::states::{Requirement, QualifierConfig, is_admin, Querier};
use crate::errors::ContractError;


pub type ExecuteResult = Result<Response, ContractError>;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    let mut response = Response::new();
    response = response.add_attribute("action", "instantiate");

    QualifierConfig {
        admin: info.sender,
        continue_option_on_fail: msg.continue_option_on_fail,
    }.save(deps.storage)?;

    Requirement {
        min_token_balances: msg.min_token_balances,
        min_luna_staking: msg.min_luna_staking,
        participation_limit: msg.participation_limit,
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    continue_option_on_fail: Option<QualifiedContinueOption>,
) -> ExecuteResult {
    if !is_admin(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new();
    response = response.add_attribute("action", "update_config");

    let mut config = QualifierConfig::load(deps.storage)?;

    if let Some(admin) = admin {
        config.admin = deps.api.addr_validate(admin.as_str())?;
        response = response.add_attribute("is_updated_admin", "true");
    }

    if let Some(continue_option_on_fail) = continue_option_on_fail {
        config.continue_option_on_fail = continue_option_on_fail;
        response = response.add_attribute("is_updated_continue_option_on_fail", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn update_requirement(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    min_token_balances: Option<Vec<(Denom, Uint128)>>,
    min_luna_staking: Option<Uint128>,
    participation_limit: Option<u64>,
) -> ExecuteResult {
    if !is_admin(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new();
    response = response.add_attribute("action", "update_requirement");

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
        response = response.add_attribute("is_updated_min_token_balances", "true");
    }

    if let Some(min_luna_staking) = min_luna_staking {
        if min_luna_staking.is_zero() {
            return Err(ContractError::Std(StdError::generic_err("Invalid input min_luna_staking")));
        }

        requirement.min_luna_staking = min_luna_staking;
        response = response.add_attribute("is_updated_min_luna_staking", "true");
    }

    if let Some(participation_limit) = participation_limit {
        requirement.participation_limit = participation_limit;
        response = response.add_attribute("is_updated_participation_limit", "true");
    }

    requirement.save(deps.storage)?;

    Ok(response)
}

pub fn qualify(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: QualificationMsg,
) -> ExecuteResult {
    let mut response = Response::new();

    response = response.add_attribute("action", "qualify");

    let campaign = deps.api.addr_validate(msg.campaign.as_str())?;
    let actor = deps.api.addr_validate(msg.actor.as_str())?;

    let requirement = Requirement::load(deps.storage)?;
    let querier = Querier::new(&deps.querier);

    let (is_valid, error_msg) = requirement.is_satisfy_requirements(&querier, &campaign, &actor)?;
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

    response = response.add_attribute("qualified_continue_option", result.continue_option.to_string());

    response = response.set_data(to_binary(&result)?);

    Ok(response)
}
