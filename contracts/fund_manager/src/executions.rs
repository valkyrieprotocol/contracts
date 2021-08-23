use cosmwasm_std::{coin, DepsMut, Env, MessageInfo, Response, StdError, to_binary, Uint128, Decimal, Addr, StdResult};
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
        admins: msg.admins.iter().map(|v| deps.api.addr_validate(v)).collect::<StdResult<Vec<Addr>>>()?,
        managing_token: deps.api.addr_validate(msg.managing_token.as_str())?,
        terraswap_router: deps.api.addr_validate(msg.terraswap_router.as_str())?,
        campaign_deposit_fee_burn_ratio: msg.campaign_deposit_fee_burn_ratio,
        campaign_deposit_fee_recipient: deps.api.addr_validate(msg.campaign_deposit_fee_recipient.as_str())?,
    }.save(deps.storage)?;

    ContractState {
        remain_allowance_amount: Uint128::zero(),
        campaign_deposit_fee_amount: Uint128::zero(),
    }.save(deps.storage)?;

    Ok(response)
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admins: Option<Vec<String>>,
    terraswap_router: Option<String>,
    campaign_deposit_fee_burn_ratio: Option<Decimal>,
    campaign_deposit_fee_recipient: Option<String>,
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
            .map(|v| deps.api.addr_validate(v))
            .collect::<StdResult<Vec<Addr>>>()?;
        response = response.add_attribute("is_updated_admins", "true");
    }

    if let Some(terraswap_router) = terraswap_router.as_ref() {
        config.terraswap_router = deps.api.addr_validate(terraswap_router.as_str())?;
        response = response.add_attribute("is_updated_terraswap_router", "true");
    }

    if let Some(campaign_deposit_fee_burn_ratio) = campaign_deposit_fee_burn_ratio {
        config.campaign_deposit_fee_burn_ratio = campaign_deposit_fee_burn_ratio;
        response = response.add_attribute("is_updated_campaign_deposit_fee_burn_ratio", "true");
    }

    if let Some(campaign_deposit_fee_recipient) = campaign_deposit_fee_recipient.as_ref() {
        config.campaign_deposit_fee_recipient = deps.api.addr_validate(campaign_deposit_fee_recipient.as_str())?;
        response = response.add_attribute("is_updated_campaign_deposit_fee_recipient", "true");
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
    let free_balance = state.load_balance(&deps.querier, &env, &config.managing_token)?.free_balance;
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

    response = response.add_attribute("address", address.to_string());
    response = response.add_attribute("amount", amount.clone());
    response = response.add_attribute("allowed_amount", allowance.allowed_amount);
    response = response.add_attribute("remain_amount", allowance.remain_amount);

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

    response = response.add_attribute("address", address.to_string());
    response = response.add_attribute("amount", amount.to_string());
    response = response.add_attribute("allowed_amount", allowance.allowed_amount);
    response = response.add_attribute("remain_amount", allowance.remain_amount);

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
        let balance = state.load_balance(&deps.querier, &env, &config.managing_token)?;

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

    response = response.add_message(message_factories::wasm_execute(
        &config.managing_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_validate(&recipient)?.to_string(),
            amount,
        },
    ));

    response = response.add_attribute("requester", info.sender.as_str());
    response = response.add_attribute("recipient", recipient);
    response = response.add_attribute("amount", amount);
    response = response.add_attribute("remain_amount", remain_amount);

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
        pair_to_terraswap_operation(pair)
    }).collect();

    let terraswap_msg = TerraswapExecuteMsg::ExecuteSwapOperations {
        operations,
        minimum_receive: None,
        to: None,
    };

    let balance = query_balance(
        &deps.querier,
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

    response = response.add_message(swap_msg);

    Ok(response)
}

fn pair_to_terraswap_operation(pair: &[Denom]) -> SwapOperation {
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
        offer_asset_info: denom_to_asset_info(left),
        ask_asset_info: denom_to_asset_info(right),
    }
}

fn denom_to_asset_info(denom: Denom) -> AssetInfo {
    match denom {
        Denom::Native(denom) => AssetInfo::NativeToken {
            denom,
        },
        Denom::Token(address) => AssetInfo::Token {
            contract_addr: address,
        },
    }
}

pub fn receive_campaign_deposit_fee(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let mut response = make_response("receive_campaign_deposit_fee");

    let mut state = ContractState::load(deps.storage)?;
    state.campaign_deposit_fee_amount += amount;

    state.save(deps.storage)?;

    response = response.add_attribute("campaign_deposit_fee_balance", state.campaign_deposit_fee_amount.to_string());

    Ok(response)
}

pub fn distribute_campaign_deposit_fee(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Option<Uint128>,
) -> ContractResult<Response> {
    let mut response = make_response("distribute_campaign_deposit_fee");

    let config = ContractConfig::load(deps.storage)?;
    let mut state = ContractState::load(deps.storage)?;

    let amount = if let Some(amount) = amount {
        state.campaign_deposit_fee_amount = state.campaign_deposit_fee_amount.checked_sub(amount)?; //check overflow
        amount
    } else {
        let amount = state.campaign_deposit_fee_amount;
        state.campaign_deposit_fee_amount = Uint128::zero();
        amount
    };

    state.save(deps.storage)?;

    let burn_amount = amount * config.campaign_deposit_fee_burn_ratio;
    let distribute_amount = amount.checked_sub(burn_amount)?;

    response = response.add_message(message_factories::wasm_execute(
        &config.managing_token,
        &Cw20ExecuteMsg::Transfer {
            recipient: config.campaign_deposit_fee_recipient.to_string(),
            amount: distribute_amount,
        },
    ));

    response = response.add_message(message_factories::wasm_execute(
        &config.managing_token,
        &Cw20ExecuteMsg::Burn {
            amount: burn_amount,
        },
    ));

    Ok(response)
}
