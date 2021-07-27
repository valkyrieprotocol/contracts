use cosmwasm_std::{Api, coin, DepsMut, Env, MessageInfo, Response, StdError, to_binary, Uint128};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::AssetInfo;
use terraswap::router::{ExecuteMsg as TerraswapExecuteMsg, SwapOperation};

use valkyrie::common::{ContractResult, Denom};
use valkyrie::cw20::query_balance;
use valkyrie::errors::ContractError;
use valkyrie::fund_manager::execute_msgs::InstantiateMsg;
use valkyrie::message_factories;
use valkyrie::utils::make_response;

use crate::states::{Allowance, ContractConfig, ContractState};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    // Execute
    let response = make_response("instantiate");

    ContractConfig {
        admins: msg.admins.iter().map(|v| deps.api.addr_validate(v).unwrap()).collect(),
        managing_token: deps.api.addr_validate(msg.managing_token.as_str())?,
        terraswap_router: deps.api.addr_validate(msg.terraswap_router.as_str())?,
    }.save(deps.storage)?;

    ContractState {
        remain_allowance_amount: Uint128::zero(),
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admins: Option<Vec<String>>,
    terraswap_router: Option<String>,
) -> ContractResult<Response> {
    // Validate
    let mut config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("update_config");

    if let Some(admins) = admins.as_ref() {
        config.admins = admins.iter()
            .map(|v| deps.api.addr_validate(v).unwrap())
            .collect();
        response.add_attribute("is_updated_admins", "true");
    }

    if let Some(terraswap_router) = terraswap_router.as_ref() {
        config.terraswap_router = deps.api.addr_validate(terraswap_router.as_str())?;
        response.add_attribute("is_updated_terraswap_router", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

pub fn increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
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

    // Execute
    let mut response = make_response("increase_allowance");

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load_or_default(deps.storage, &address)?;

    allowance.increase(amount.clone());
    allowance.save(deps.storage)?;

    state.remain_allowance_amount += amount;
    state.save(deps.storage)?;

    response.add_attribute("address", address.to_string());
    response.add_attribute("amount", amount.clone());
    response.add_attribute("allowed_amount", allowance.allowed_amount);
    response.add_attribute("remain_amount", allowance.remain_amount);

    Ok(response)
}

pub fn decrease_allowance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut response = make_response("decrease_allowance");

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

    allowance.decrease(amount)?;
    allowance.save_or_delete(deps.storage)?;

    state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
    state.save(deps.storage)?;

    response.add_attribute("address", address.to_string());
    response.add_attribute("amount", amount.to_string());
    response.add_attribute("allowed_amount", allowance.allowed_amount);
    response.add_attribute("remain_amount", allowance.remain_amount);

    Ok(response)
}

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    // Validate
    let config = ContractConfig::load(deps.storage)?;
    let mut state = ContractState::load(deps.storage)?;

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // Execute
    let mut response = make_response("transfer");

    let remain_amount = if config.is_admin(&info.sender) {
        let balance = state.load_balance(&deps.querier, deps.api, &env, &config.managing_token)?;

        if balance.free_balance < amount {
            return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
        }

        balance.free_balance
    } else {
        let allowance = Allowance::may_load(deps.storage, &info.sender)?;

        if let Some(mut allowance) = allowance {
            if allowance.remain_amount < amount {
                return Err(ContractError::ExceedLimit {});
            }

            allowance.remain_amount = allowance.remain_amount.checked_sub(amount)?;
            allowance.save_or_delete(deps.storage)?;

            state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
            state.save(deps.storage)?;

            allowance.remain_amount
        } else {
            return Err(ContractError::Unauthorized {});
        }
    };

    response.add_message(message_factories::wasm_execute(
        &config.managing_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_validate(&recipient)?.to_string(),
            amount,
        },
    ));

    response.add_attribute("requester", info.sender.as_str());
    response.add_attribute("recipient", recipient);
    response.add_attribute("amount", amount);
    response.add_attribute("remain_amount", remain_amount);

    Ok(response)
}

pub fn swap(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    denom: Denom,
    amount: Option<Uint128>,
    route: Option<Vec<Denom>>,
) -> ContractResult<Response> {
    // Validate
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

    // Execute
    let mut response = make_response("swap");

    let operations: Vec<SwapOperation> = route.windows(2).map(|pair| {
        pair_to_terraswap_operation(pair, deps.api)
    }).collect();

    let terraswap_msg = TerraswapExecuteMsg::ExecuteSwapOperations {
        operations,
        minimum_receive: None,
        to: None,
    };

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

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let swap_msg = match denom {
        Denom::Native(denom) => {
            message_factories::wasm_execute_with_funds(
                &config.terraswap_router,
                vec![coin(amount.u128(), denom)],
                &terraswap_msg,
            )
        }
        Denom::Token(address) => {
            message_factories::wasm_execute(
                &deps.api.addr_validate(&address)?,
                &Cw20ExecuteMsg::Send {
                    contract: config.terraswap_router.to_string(),
                    msg: to_binary(&terraswap_msg)?,
                    amount,
                },
            )
        }
    };

    response.add_message(swap_msg);

    Ok(response)
}

fn pair_to_terraswap_operation(pair: &[Denom], api: &dyn Api) -> SwapOperation {
    let left = pair[0].clone();
    let right = pair[1].clone();

    if let Denom::Native(left_denom) = left.clone() {
        if let Denom::Native(right_denom) = right.clone() {
            return SwapOperation::NativeSwap {
                offer_denom: left_denom,
                ask_denom: right_denom,
            };
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
