use cosmwasm_std::{Addr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, WasmMsg};
use cw20::{Cw20ExecuteMsg, Denom};
use valkyrie::proxy::execute_msgs::{SwapOperation};

use valkyrie::cw20::query_balance;
use valkyrie::errors::ContractError;
use valkyrie::proxy::asset::{Asset, AssetInfo};
use crate::astroport::msgs::AstroportPairExecuteMsg;

use crate::astroport::queries::{query_pair_info};

pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    factory: Addr,
    operation: SwapOperation,
    to: Option<String>,
    max_spread: Option<Decimal>,
) -> Result<Response, ContractError> {
    let message = match operation {
        SwapOperation::Swap {
            offer_asset_info,
            ask_asset_info,
        } => {
            let pair_info = query_pair_info(
                deps.as_ref(),
                &factory,
                &[offer_asset_info.clone(), ask_asset_info],
            )?;

            let amount = match &offer_asset_info {
                AssetInfo::NativeToken { denom } => {
                    query_balance(&deps.querier, Denom::Native(denom.to_string()), env.contract.address)?
                }
                AssetInfo::Token { contract_addr } => {
                    query_balance(&deps.querier, Denom::Cw20(deps.api.addr_validate(contract_addr.as_str())?), env.contract.address)?
                }
            };

            let offer_asset = Asset {
                info: offer_asset_info,
                amount,
            };

            asset_into_swap_msg(
                pair_info.contract_addr,
                offer_asset,
                max_spread,
                to,
            )?
        }
    };

    Ok(Response::new().add_message(message))
}

fn asset_into_swap_msg(
    pair_contract: String,
    offer_asset: Asset,
    max_spread: Option<Decimal>,
    to: Option<String>,
) -> StdResult<CosmosMsg> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_contract,
            funds: vec![Coin {
                denom,
                amount: offer_asset.amount,
            }],
            msg: to_binary(&AstroportPairExecuteMsg::Swap {
                offer_asset,
                belief_price: None,
                max_spread,
                to,
            })?,
        })),
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract,
                amount: offer_asset.amount,
                msg: to_binary(&AstroportPairExecuteMsg::Swap {
                    offer_asset,
                    belief_price: None,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}