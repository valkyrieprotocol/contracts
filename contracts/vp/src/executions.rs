use astroport::asset::{Asset, AssetInfo};
use astroport::querier::query_token_balance;
use astroport::router::{ExecuteMsg as AstroExecuteMsg, SwapOperation};
use cosmwasm_std::{Addr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, to_binary, Uint128, WasmMsg};
use crate::msg::{ExecuteMsg};
use crate::state::{Config, State, SwapRatio, SwapState};
use cw20::Cw20ExecuteMsg;
use cw20_base::ContractError;

static UST:&str = "uusd";

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    admin: Option<String>,
    whitelist: Option<Vec<String>>,
    offer_token: Option<String>,
    base_swap_ratio: Option<Decimal>,
    custom_swap_ratio: Option<Vec<SwapRatio>>,
    router: Option<String>,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    response = response.add_attribute("action", "update_config");

    let mut config = Config::load(deps.storage)?;

    if let Some(admin) = admin.as_ref() {
        Config::save_admin_nominee(deps.storage, &deps.api.addr_validate(admin)?)?;
        response = response.add_attribute("is_updated_admin_nominee", "true");
    }

    if let Some(whitelist) = whitelist {
        config.whitelist = whitelist.iter()
            .map(|item| deps.api.addr_validate(item.as_str()))
            .collect::<StdResult<Vec<Addr>>>()?;
        response = response.add_attribute("is_updated_whitelist", "true");
    }

    if let Some(offer_token) = offer_token {
        config.offer_token = deps.api.addr_validate(offer_token.as_str())?;
        response = response.add_attribute("is_updated_offer_token", "true");
    }

    if let Some(base_swap_ratio) = base_swap_ratio {
        config.base_swap_ratio = base_swap_ratio;
        response = response.add_attribute("is_updated_base_swap_ratio", "true");
    }

    if let Some(custom_swap_ratio) = custom_swap_ratio {
        config.custom_swap_ratio = custom_swap_ratio;
        response = response.add_attribute("is_updated_custom_swap_ratio", "true");
    }

    if let Some(router) = router {
        config.router = deps.api.addr_validate(router.as_str())?;
        response = response.add_attribute("is_updated_router", "true");
    }

    config.save(deps.storage)?;
    Ok(response)
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender_raw: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    response = response.add_attribute("action", "mint");

    let sender = deps.api.addr_validate(sender_raw.as_str())?;
    let config = Config::load(deps.storage)?;

    let ratio = config.get_swap_ratio(&sender);
    let mint_amount = amount * ratio;

    let mut state = State::load_or_default(deps.storage)?;
    state.cumulative_offer_amount += amount;
    state.cumulative_mint_amount += mint_amount;
    state.save(deps.storage)?;

    let mut swap_state = SwapState::load_or_default(deps.storage, ratio)?;
    swap_state.cumulative_offer_amount += amount;
    swap_state.save(deps.storage)?;

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.offer_token.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount,
        })?,
    }));

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: sender.to_string(),
            amount: mint_amount,
        })?,
    }));

    response = response.add_attribute("burn_amount", amount.to_string());
    response = response.add_attribute("mint_amount", mint_amount.to_string());

    Ok(response)
}

pub fn approve_admin_nominee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Execute
    let mut response = Response::new();
    response = response.add_attribute("action", "approve_admin_nominee");

    if let Some(admin_nominee) = Config::may_load_admin_nominee(deps.storage)? {
        if admin_nominee != info.sender {
            return Err(ContractError::Std(StdError::generic_err("It is not admin nominee")));
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = Config::load(deps.storage)?;
    config.admin = info.sender;
    response = response.add_attribute("is_updated_admin", "true");

    config.save(deps.storage)?;

    Ok(response)
}

pub fn mint_from_uusd(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut uusd_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: UST.to_string()
        },
        amount: Uint128::zero()
    };

    for fund in info.funds.iter() {
        if fund.denom == UST.to_string() {
            uusd_asset.amount += fund.amount;
        } else {
            return Err(ContractError::Std(StdError::generic_err("allowed uusd only.".to_string())));
        }
    }

    if uusd_asset.amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("uusd amount is zero".to_string())));
    }

    let config = Config::load(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "mint_from_uusd");

    // UST => VKR
    response.messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.router.to_string(),
        funds: vec![
            Coin {
                denom: UST.to_string(),
                amount: uusd_asset.amount - uusd_asset.compute_tax(&deps.querier)?
            }
        ],
        msg: to_binary(&AstroExecuteMsg::ExecuteSwapOperations {
            // AstroExecuteMsg::ExecuteSwapOperations => OK
            // AstroExecuteMsg::ExecuteSwapOperation  => Unauthorized
            operations: vec![
                SwapOperation::AstroSwap {
                    offer_asset_info: uusd_asset.info,
                    ask_asset_info: AssetInfo::Token {
                        contract_addr: config.offer_token.clone()
                    }
                }
            ],
            minimum_receive: None,
            to: Some(env.contract.address.clone())
        })?,
    })));

    let exist_balance = query_token_balance(&deps.querier, config.offer_token, env.contract.address.clone())?;

    response.messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::MintFromUusdHook {
            burner: info.sender.to_string(),
            exist_balance,
        })?,
    })));

    Ok(response)
}

pub fn mint_from_uusd_hook(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    burner: String,
    exist_balance: Uint128,
) -> Result<Response, ContractError> {
    let contract_address = env.clone().contract.address;

    if info.sender.to_string() != contract_address.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    let config = Config::load(deps.storage)?;
    let now_balance = query_token_balance(&deps.querier, config.offer_token, contract_address.clone())?;
    let burn_amount = now_balance - exist_balance;

    //send VKR to this(VP token contract). to burn VKR and mint VP
    mint(deps, env, info, burner, burn_amount)
}