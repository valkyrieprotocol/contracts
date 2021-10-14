use cosmwasm_std::{Deps, Env, Uint128};

use valkyrie::common::ContractResult;

use crate::states::{ContractConfig, ContractState, Distribution};
use valkyrie::distributor::query_msgs::{StateResponse, DistributionsResponse, DistributionResponse, ContractConfigResponse};
use valkyrie::cw20::query_cw20_balance;

pub fn get_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let config = ContractConfig::load(deps.storage)?;

    Ok(ContractConfigResponse {
        admins: config.admins.iter().map(|v| v.to_string()).collect(),
        managing_token: config.managing_token.to_string(),
    })
}

pub fn get_state(deps: Deps, env: Env) -> ContractResult<StateResponse> {
    let config = ContractConfig::load(deps.storage)?;
    let state = ContractState::load(deps.storage)?;
    let balance = query_cw20_balance(
        &deps.querier,
        &config.managing_token,
        &env.contract.address,
    )?;

    Ok(StateResponse {
        balance,
        locked_amount: state.locked_amount,
        distributed_amount: state.distributed_amount,
        free_amount: balance.checked_sub(state.locked_amount)?,
    })
}

pub fn get_distributions(deps: Deps, env: Env) -> ContractResult<DistributionsResponse> {
    let distributions = Distribution::load_all(deps.storage)?.iter()
        .map(|d| {
            let released_amount = d.released_amount(env.block.height);

            DistributionResponse {
                id: d.id,
                start_height: d.start_height,
                end_height: d.end_height,
                recipient: d.recipient.to_string(),
                amount: d.amount,
                released_amount,
                distributable_amount: released_amount.checked_sub(d.distributed_amount)
                    .unwrap_or(Uint128::zero()),
                distributed_amount: d.distributed_amount,
            }
        })
        .collect();

    Ok(DistributionsResponse {
        distributions,
    })
}
