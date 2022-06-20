use cosmwasm_std::{Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, Uint128, WasmMsg};
use crate::state::{Config, State, SwapRatio, SwapState};
use cw20::Cw20ExecuteMsg;
use cw20_base::ContractError;

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