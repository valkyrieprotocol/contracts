use std::collections::{HashSet};
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Uint128, };
use cw20::{Denom};
use valkyrie::common::{ContractResult};

use crate::states::*;

use valkyrie::cw20::query_balance;
use valkyrie::errors::ContractError;
use valkyrie::proxy::asset::{AssetInfo};
use valkyrie::proxy::execute_msgs::{DexInfo, DexType, find_dex_factory, SwapOperation};
use valkyrie::proxy::query_msgs::SimulateSwapOperationsResponse;
use crate::executions::{MAX_SWAP_OPERATIONS};

pub fn get_config(deps: Deps, _env: Env) -> ContractResult<Config> {
    let config = Config::load(deps.storage)?;
    Ok(config)
}

pub fn simulate_swap_operations(
    deps: Deps,
    offer_amount: Uint128,
    operations: Vec<SwapOperation>,
) -> ContractResult<SimulateSwapOperationsResponse> {
    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(ContractError::Std(StdError::generic_err("must provide options")));
    }

    if operations_len > MAX_SWAP_OPERATIONS {
        return Err(ContractError::Std(StdError::generic_err("swap limit exceed")));
    }

    assert_operations(&operations)?;

    let mut offer_amount = offer_amount;
    for operation in operations.into_iter() {
        let offer_asset_info = operation.get_offer_asset_info();
        let ask_asset_info = operation.get_target_asset_info();

        let dex_info = get_largest_pool(deps, [offer_asset_info.clone(), ask_asset_info.clone()])?;

        match dex_info.dex_type {
            DexType::Astroport => {
                offer_amount = crate::astroport::queries::simulate_swap_operation(
                    deps,
                    &dex_info.factory,
                    offer_asset_info.clone(),
                    ask_asset_info.clone(),
                    offer_amount
                )?;
            }
        };
    }

    Ok(SimulateSwapOperationsResponse {
        amount: offer_amount
    })
}

pub fn get_largest_pool(
    deps: Deps,
    pairs: [AssetInfo; 2],
) -> StdResult<DexInfo> {

    let config = Config::load(deps.storage)?;

    if let Some(fixed_dex) = config.fixed_dex {
        if let Some(dex_info) = find_dex_factory(config.dex_list, fixed_dex) {
            Ok(dex_info)
        } else {
            return Err(StdError::generic_err("unavailable factory."))
        }
    } else {
        let mut pool_size = Uint128::zero();
        let mut selected = config.dex_list[0].clone();

        for dex_info in config.dex_list {
            match dex_info.dex_type {
                DexType::Astroport => {
                    let size = crate::astroport::queries::query_pool_size(deps, dex_info.factory.clone(), pairs.clone())?;
                    if size > pool_size {
                        pool_size = size;
                        selected = dex_info.clone();
                    }
                }
            }
        }

        Ok(selected)
    }
}

pub fn query_pool(
    deps: Deps,
    asset_info: AssetInfo,
    pool_addr: Addr,
) -> StdResult<Uint128> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => query_balance(
            &deps.querier,
            Denom::Cw20(deps.api.addr_validate(contract_addr.as_str())?),
            pool_addr,
        ),
        AssetInfo::NativeToken { denom, .. } => query_balance(
            &deps.querier,
            Denom::Native(denom),
            pool_addr,
        )
    }
}

fn assert_operations(
    operations: &[SwapOperation]
) -> Result<(), StdError> {
    let mut ask_asset_map: HashSet<String> = HashSet::new();
    for operation in operations {
        let (offer_asset, ask_asset) = match operation {
            SwapOperation::Swap {
                offer_asset_info,
                ask_asset_info,
            } => (
                offer_asset_info,
                ask_asset_info,
            ),
        };

        ask_asset_map.remove(&offer_asset.to_string());
        ask_asset_map.insert(ask_asset.to_string());
    }

    if ask_asset_map.len() != 1 {
        return Err(StdError::generic_err("invalid operations; multiple output token").into());
    }

    Ok(())
}