use cosmwasm_std::{Deps, Env, Uint128};
use valkyrie_qualifier::{QualificationMsg, QualificationResult, QualifiedContinueOption};
use crate::errors::ContractError;
use crate::states::{Requirement, QualifierConfig, Querier, Collateral};


pub type QueryResult<T> = Result<T, ContractError>;

pub fn qualify(
    deps: Deps,
    env: Env,
    msg: QualificationMsg,
) -> QueryResult<QualificationResult> {
    let actor = deps.api.addr_validate(msg.actor.as_str())?;

    let requirement = Requirement::load(deps.storage)?;
    let querier = Querier::new(&deps.querier);

    let collateral_balance = if requirement.require_collateral() {
        Collateral::load_or_new(deps.storage, &actor)?.balance(env.block.height)?
    } else {
        Uint128::zero()
    };

    let (is_valid, error_msg) = requirement.is_satisfy_requirements(&querier, &actor, collateral_balance)?;

    if !is_valid {
        let config = QualifierConfig::load(deps.storage)?;

        return Ok(QualificationResult {
            continue_option: config.continue_option_on_fail,
            reason: Some(error_msg),
        });
    }

    Ok(QualificationResult {
        continue_option: QualifiedContinueOption::Eligible,
        reason: None,
    })
}

pub fn requirement(
    deps: Deps,
    _env: Env,
) -> QueryResult<Requirement> {
    Ok(Requirement::load(deps.storage)?)
}
