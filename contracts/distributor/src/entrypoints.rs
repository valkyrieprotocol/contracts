#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::distributor::execute_msgs::{ExecuteMsg, InstantiateMsg};
use valkyrie::distributor::query_msgs::QueryMsg;

use crate::{
    executions::{add_distributor, remove_distributor, spend},
    queries::{get_contract_config, get_distributor_info, get_distributor_infos},
    states::ContractConfig,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = ContractConfig {
        governance: deps.api.addr_validate(&msg.governance)?,
        token_contract: deps.api.addr_validate(&msg.token_contract)?,
    };

    config.save(deps.storage)?;

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
        ExecuteMsg::AddDistributor {
            distributor,
            spend_limit,
        } => add_distributor(deps, env, info, distributor, spend_limit),
        ExecuteMsg::RemoveDistributor { distributor } => {
            remove_distributor(deps, env, info, distributor)
        }
        ExecuteMsg::Spend { recipient, amount } => spend(deps, env, info, recipient, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::ContractConfig {} => to_binary(&get_contract_config(deps, env)?),
        QueryMsg::DistributorInfo { distributor } => {
            to_binary(&get_distributor_info(deps, env, distributor)?)
        }
        QueryMsg::DistributorInfos {
            start_after,
            limit,
            order_by,
        } => to_binary(&get_distributor_infos(
            deps,
            env,
            start_after,
            limit,
            order_by,
        )?),
    }?;

    Ok(result)
}
