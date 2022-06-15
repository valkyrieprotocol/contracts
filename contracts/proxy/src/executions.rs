use std::collections::{HashSet};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, Uint128, WasmMsg};
use valkyrie::proxy::execute_msgs::{DexInfo, DexType, ExecuteMsg, SwapOperation};

use valkyrie::common::{ContractResult};
use valkyrie::errors::ContractError;
use valkyrie::proxy::asset::{AssetInfo};
use valkyrie::proxy::execute_msgs::InstantiateMsg;
use valkyrie::utils::{make_response};

use crate::queries::{get_largest_pool, query_pool};
use crate::states::*;

pub const MAX_SWAP_OPERATIONS: usize = 50;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    Config {
        admin: info.sender,
        fixed_dex: None,
        dex_list: vec![
            DexInfo {
                dex_type: DexType::Astroport,
                factory: deps.api.addr_validate(msg.astroport_factory.as_str())?,
            }
        ],
    }.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fixed_dex: Option<DexType>,
) -> ContractResult<Response> {
    // Validate
    let mut config = Config::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_config");

    if let Some(fixed_dex) = fixed_dex {
        config.fixed_dex = Some(fixed_dex);
        response = response.add_attribute("is_updated_fixed_dex", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn clear_fixed_dex(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response> {
    // Execute
    let mut config = Config::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    config.fixed_dex = None;
    config.save(deps.storage)?;

    let response = make_response("clear_fixed_dex");
    Ok(response)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_swap_operations(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    operations: Vec<SwapOperation>,
    minimum_receive: Option<Uint128>,
    to: Option<String>,
    max_spread: Option<Decimal>,
) -> ContractResult<Response> {
    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(ContractError::Std(StdError::generic_err("must provide operations")));
    }

    if operations_len > MAX_SWAP_OPERATIONS {
        return Err(ContractError::Std(StdError::generic_err("swap limit exceed")));
    }

    // Assert the operations are properly set
    assert_operations(&operations)?;


    let to = to.unwrap_or(sender.to_string());
    let target_asset_info = operations.last().unwrap().get_target_asset_info();

    let mut messages = operations
        .into_iter()
        .enumerate()
        .map(|(operation_index, op)| {
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: op,
                    to: if operation_index == operations_len - 1 {
                        Some(to.to_string())
                    } else {
                        None
                    },
                    max_spread,
                })?,
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    // Execute minimum amount assertion
    if let Some(minimum_receive) = minimum_receive {
        let receiver_balance = query_pool(deps.as_ref(), target_asset_info.clone(), deps.api.addr_validate(to.as_str())?)?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::AssertMinimumReceive {
                asset_info: target_asset_info.clone(),
                prev_balance: receiver_balance,
                minimum_receive,
                receiver: to.to_string(),
            })?,
        }));
    }

    Ok(Response::new().add_messages(messages))
}

fn assert_operations(operations: &[SwapOperation]) -> StdResult<()> {
    let mut ask_asset_map: HashSet<String> = HashSet::new();
    for operation in operations {
        let (offer_asset, ask_asset) = match operation {
            SwapOperation::Swap {
                offer_asset_info,
                ask_asset_info,
            } => (offer_asset_info.clone(), ask_asset_info.clone()),
        };

        ask_asset_map.remove(&offer_asset.to_string());
        ask_asset_map.insert(ask_asset.to_string());
    }

    if ask_asset_map.len() != 1 {
        return Err(StdError::generic_err("invalid operations; multiple output token").into());
    }

    Ok(())
}

pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation: SwapOperation,
    to: Option<String>,
    max_spread: Option<Decimal>,
) -> ContractResult<Response> {
    let offer_asset_info = operation.get_offer_asset_info();
    let ask_asset_info = operation.get_target_asset_info();
    let lagest_pool = get_largest_pool(deps.as_ref(), [offer_asset_info, ask_asset_info])?;

    let response = match lagest_pool.dex_type {
        DexType::Astroport => {
            crate::astroport::executions::execute_swap_operation(
                deps,
                env,
                info,
                lagest_pool.factory.clone(),
                operation.clone(),
                to,
                max_spread
            )
        }
    }?;

    Ok(response)
}

pub fn assert_minimum_receive(
    deps: Deps,
    asset_info: AssetInfo,
    prev_balance: Uint128,
    minimum_receive: Uint128,
    receiver: Addr,
) -> ContractResult<Response> {
    let receiver_balance = query_pool(deps, asset_info, receiver)?;
    let swap_amount = receiver_balance.checked_sub(prev_balance)?;

    if swap_amount < minimum_receive {
        Err(ContractError::Std(StdError::generic_err(format!("assertion minimum receive. receive: {0}, amount: {1}", minimum_receive, swap_amount))))
    } else {
        Ok(Response::default())
    }
}