use cosmwasm_std::{attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg, StdError, Api, coin};
use cw20::Cw20ExecuteMsg;

use valkyrie::common::{ContractResult, Denom};
use valkyrie::errors::ContractError;

use terraswap::asset::AssetInfo;
use terraswap::router::{SwapOperation, ExecuteMsg as TerraswapExecuteMsg};
use valkyrie::cw20::query_balance;
use crate::states::{ContractConfig, Allowance, ContractState};
use valkyrie::fund_manager::execute_msgs::InstantiateMsg;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    ContractConfig {
        admins: msg.admins.iter().map(|v| deps.api.addr_validate(v).unwrap()).collect(),
        managing_token: deps.api.addr_validate(msg.managing_token.as_str())?,
        terraswap_router: deps.api.addr_validate(msg.terraswap_router.as_str())?,
    }.save(deps.storage)?;

    ContractState {
        remain_allowance_amount: Uint128::zero(),
    }.save(deps.storage)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    terraswap_router: Option<String>,
) -> ContractResult<Response> {
    let mut config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(terraswap_router) = terraswap_router {
        config.terraswap_router = deps.api.addr_validate(terraswap_router.as_str())?;
    }

    config.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "update_config"),
        ],
        data: None,
    })
}

pub fn increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> ContractResult<Response> {
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {})
    }

    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut state = ContractState::load(deps.storage)?;
    let free_balance = state.load_balance(&deps.querier, deps.api, &env, &config.managing_token)?.free_balance;
    if free_balance < amount {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load_or_default(deps.storage, &address)?;

    allowance.increase(amount.clone());
    allowance.save(deps.storage)?;

    state.remain_allowance_amount += amount;
    state.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "increase_allowance"),
            attr("address", address.to_string()),
            attr("amount", amount.clone()),
            attr("allowed_amount", allowance.allowed_amount),
            attr("remain_amount", allowance.remain_amount),
        ],
        data: None,
    })
}

pub fn decrease_allowance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load(deps.storage, &address)?;
    let mut state = ContractState::load(deps.storage)?;

    let amount = if let Some(amount) = amount {
        if allowance.remain_amount < amount {
            return Err(ContractError::Std(StdError::generic_err("Insufficient remain amount")));
        } else {
            amount
        }
    } else {
        allowance.remain_amount.clone()
    };

    if amount < allowance.remain_amount {
        state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
        allowance.decrease(amount)?;
        allowance.save(deps.storage)?;
    } else {
        state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(allowance.remain_amount)?;
        allowance.remove(deps.storage);
    }
    state.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "decrease_allowance"),
            attr("address", address.to_string()),
            attr("amount", amount.to_string()),
            attr("allowed_amount", allowance.allowed_amount),
            attr("remain_amount", allowance.remain_amount),
        ],
        data: None,
    })
}

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let config = ContractConfig::load(deps.storage)?;
    let mut state = ContractState::load(deps.storage)?;

    if config.is_admin(&info.sender) {
        let balance = state.load_balance(&deps.querier, deps.api, &env, &config.managing_token)?;
        if balance.free_balance < amount {
            return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
        }
    } else {
        let allowance = Allowance::may_load(deps.storage, &info.sender)?;

        if let Some(mut allowance) = allowance {
            if allowance.remain_amount < amount {
                return Err(ContractError::ExceedLimit {});
            }

            allowance.remain_amount = allowance.remain_amount.checked_sub(amount)?;
            state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
            state.save(deps.storage)?;
            if allowance.remain_amount.is_zero() {
                allowance.remove(deps.storage);
            } else {
                allowance.save(deps.storage)?;
            }
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.managing_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: deps.api.addr_validate(&recipient)?.to_string(),
                amount,
            })?,
            send: vec![],
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "spend"),
            attr("requester", info.sender.as_str()),
            attr("recipient", recipient),
            attr("amount", amount),
        ],
        data: None,
    })
}

pub fn swap(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
    route: Option<Vec<Denom>>,
) -> ContractResult<Response> {
    let config = ContractConfig::load(deps.storage)?;
    let token_denom = Denom::Token(config.managing_token.to_string());
    let route = route.unwrap_or_else(|| vec![denom.clone(), token_denom.clone()]);

    if route.len() < 2 || *route.first().unwrap() != denom || *route.last().unwrap() != token_denom {
        return Err(ContractError::Std(StdError::generic_err(
            format!(
                "route must start with '{}' and end with '{}'",
                denom.to_string(), token_denom.to_string(),
            )
        )));
    }

    let operations: Vec<SwapOperation> = route.windows(2).map(|pair| {
        pair_to_terraswap_operation(pair, deps.api)
    }).collect();

    let terraswap_msg_binary = to_binary(&TerraswapExecuteMsg::ExecuteSwapOperations {
        operations,
        minimum_receive: None,
        to: None,
    })?;

    let balance = query_balance(
        &deps.querier,
        deps.api,
        denom.to_cw20(deps.api),
        env.contract.address.clone(),
    )?;
    let amount = if let Some(amount) = amount {
        if amount > balance {
            return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
        } else {
            amount
        }
    } else {
        balance
    };

    let swap_msg = match denom {
        Denom::Native(denom) => {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.terraswap_router.to_string(),
                send: vec![coin(amount.u128(), denom)],
                msg: terraswap_msg_binary,
            })
        }
        Denom::Token(address) => {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: address,
                send: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: config.terraswap_router.to_string(),
                    msg: Some(terraswap_msg_binary),
                    amount,
                }).unwrap(),
            })
        }
    };

    Ok(Response {
        submessages: vec![],
        messages: vec![swap_msg],
        attributes: vec![],
        data: None,
    })
}

fn pair_to_terraswap_operation(pair: &[Denom], api: &dyn Api) -> SwapOperation {
    let left = pair[0].clone();
    let right = pair[1].clone();

    if let Denom::Native(left_denom) = left.clone() {
        if let Denom::Native(right_denom) = right.clone() {
            return SwapOperation::NativeSwap {
                offer_denom: left_denom,
                ask_denom: right_denom,
            }
        }
    }

    SwapOperation::TerraSwap {
        offer_asset_info: denom_to_asset_info(left, api),
        ask_asset_info: denom_to_asset_info(right, api),
    }
}

fn denom_to_asset_info(denom: Denom, api: &dyn Api) -> AssetInfo {
    match denom {
        Denom::Native(denom) => AssetInfo::NativeToken {
            denom,
        },
        Denom::Token(address) => AssetInfo::Token {
            contract_addr: api.addr_validate(address.as_str()).unwrap(),
        },
    }
}
