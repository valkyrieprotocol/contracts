use cosmwasm_std::{Deps, Env};
use valkyrie_qualifier::{QualificationMsg, QualificationResult, QualifiedContinueOption};
use crate::errors::ContractError;
use crate::states::{QualificationRequirement, QualifierConfig, Querier};
use cw20::Denom;


pub type QueryResult<T> = Result<T, ContractError>;

pub fn qualify(
    deps: Deps,
    _env: Env,
    msg: QualificationMsg,
) -> QueryResult<QualificationResult> {
    let actor = deps.api.addr_validate(msg.actor.as_str())?;

    let config = QualifierConfig::load(deps.storage)?;
    let requirement = QualificationRequirement::load(deps.storage)?;

    let querier = Querier::new(&deps.querier);

    let current_luna_staking_amount = querier.load_luna_staking_amount(&actor)?;
    if current_luna_staking_amount < requirement.min_luna_staking {
        return Ok(QualificationResult {
            continue_option: config.continue_option_on_fail,
            reason: Some(format!(
                "Insufficient luna staking amount(required: {}, current: {})",
                requirement.min_luna_staking.to_string(),
                current_luna_staking_amount.to_string(),
            )),
        })
    }

    for (denom, min_balance) in requirement.min_token_balances.iter() {
        let current_balance = querier.load_balance(denom, &actor)?;

        if current_balance < *min_balance {
            return Ok(QualificationResult {
                continue_option: config.continue_option_on_fail,
                reason: Some(format!(
                    "Insufficient token({}) balance (required: {}, current: {})",
                    denom_to_string(denom),
                    min_balance.to_string(),
                    current_balance.to_string(),
                )),
            })
        }
    }

    Ok(QualificationResult {
        continue_option: QualifiedContinueOption::Eligible,
        reason: None,
    })
}

fn denom_to_string(denom: &Denom) -> String {
    match denom {
        Denom::Native(denom) => denom.to_string(),
        Denom::Cw20(address) => address.to_string(),
    }
}
