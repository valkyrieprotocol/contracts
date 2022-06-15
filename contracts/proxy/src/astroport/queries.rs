use cosmwasm_std::{Addr, Deps, StdError, StdResult, Uint128};
use cw20::{Denom};

use valkyrie::cw20::query_balance;
use valkyrie::proxy::asset::{Asset, AssetInfo};
use crate::astroport::msgs::{AstroportPairInfo, AstroportSimulationResponse, AstroportFactoryQueryMsg, AstroportPairQueryMsg};

pub fn simulate_swap_operation(
    deps: Deps,
    factory: &Addr,
    offer_asset_info: AssetInfo,
    ask_asset_info: AssetInfo,
    offer_amount: Uint128,
) -> Result<Uint128, StdError> {
    let pair_info = query_pair_info(
        deps,
        factory,
        &[offer_asset_info.clone(), ask_asset_info.clone()],
    )?;

    let res: AstroportSimulationResponse = deps.querier.query_wasm_smart(
        pair_info.contract_addr,
            &AstroportPairQueryMsg::Simulation {
                offer_asset: Asset {
                    info: offer_asset_info.clone(),
                    amount: offer_amount,
                },
            },
    )?;

    Ok(res.return_amount)
}



pub fn query_pair_info(
    deps: Deps,
    factory_contract: &Addr,
    asset_infos: &[AssetInfo; 2],
) -> StdResult<AstroportPairInfo> {
    deps.querier.query_wasm_smart(
        factory_contract.to_string(),
        &AstroportFactoryQueryMsg::Pair {
            asset_infos: asset_infos.clone(),
        },
    )
}

pub fn query_pool_size(
    deps: Deps,
    factory: Addr,
    asset_infos: [AssetInfo; 2],
) -> StdResult<Uint128> {
    let pair_info = query_pair_info(deps, &factory, &asset_infos)?;
    let pool_addr = deps.api.addr_validate(pair_info.contract_addr.as_str())?;

    let mut pool = vec![Uint128::zero(), Uint128::zero()];

    for index in 0..asset_infos.to_vec().len() {
        let asset_info = &asset_infos.to_vec()[index];
        pool[index] = match asset_info {
            AssetInfo::Token { contract_addr } => query_balance(&deps.querier, Denom::Cw20(deps.api.addr_validate(contract_addr.as_str())?), pool_addr.clone()),
            AssetInfo::NativeToken { denom } => query_balance(&deps.querier, Denom::Native(denom.to_string()), pool_addr.clone())
        }?.clone();
    }

    let size = pool[0] * pool[1];
    Ok(size)
}