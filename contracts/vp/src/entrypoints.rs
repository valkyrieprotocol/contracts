#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, from_binary, Addr, };
use cw20::{Cw20ReceiveMsg, MinterResponse};

use crate::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::queries::{query_config, query_state, query_swap_state};
use crate::state::{Config};
use cw20_base::ContractError;
use cw2::set_contract_version;
use crate::migrations;

const CONTRACT_NAME: &str = "valkyrian-pass-cw20-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut response = Response::new();
    response = response.add_attribute("action", "instantiate");

    Config {
        admin: deps.api.addr_validate(msg.admin.as_str())?,
        whitelist: msg.whitelist.iter()
            .map(|item| deps.api.addr_validate(item.as_str()))
            .collect::<StdResult<Vec<Addr>>>()?,
        offer_token: deps.api.addr_validate(msg.offer_token.as_str())?,
        base_swap_ratio: msg.base_swap_ratio,
        custom_swap_ratio: msg.custom_swap_ratio,
        router: deps.api.addr_validate(msg.router.as_str())?,
    }.save(deps.storage)?;

    cw20_base::contract::instantiate(
        deps,
        env.clone(),
        info,
        cw20_base::msg::InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: msg.initial_balances,
            marketing: msg.marketing,
            mint: Some(MinterResponse {
                minter: env.contract.address.to_string(),
                cap: None,
            }),
    })?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Burn { amount } => cw20_base::contract::execute_burn(deps, env, info, amount),
        ExecuteMsg::Transfer { recipient, amount } => {
            let config = Config::load(deps.storage)?;
            if !config.is_whitelisted(&info.sender) {
                return Err(ContractError::Unauthorized {});
            }
            cw20_base::contract::execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Send { contract, amount, msg } => {
            let config = Config::load(deps.storage)?;
            if !config.is_whitelisted(&info.sender) {
                return Err(ContractError::Unauthorized {});
            }
            cw20_base::contract::execute_send(deps, env, info, contract, amount, msg)
        }
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            let config = Config::load(deps.storage)?;
            if !config.is_whitelisted(&info.sender) {
                return Err(ContractError::Unauthorized {});
            }

            cw20_base::allowances::execute_transfer_from(deps, env, info, owner, recipient, amount)
        },
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => {
            let config = Config::load(deps.storage)?;
            if !config.is_whitelisted(&info.sender) {
                return Err(ContractError::Unauthorized {});
            }

            cw20_base::allowances::execute_send_from(deps, env, info, owner, contract, amount, msg)
        },
        ExecuteMsg::BurnFrom { owner, amount } => cw20_base::allowances::execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::Mint { recipient, amount } => {
            if env.contract.address != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            cw20_base::contract::execute_mint(deps, env, info, recipient, amount)
        },
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => {
            cw20_base::contract::execute_update_marketing(deps, env, info, project, description, marketing)
        },
        ExecuteMsg::UploadLogo(logo) => {
            cw20_base::contract::execute_upload_logo(deps, env, info, logo)
        },
        ExecuteMsg::UpdateConfig {
            admin,
            whitelist,
            offer_token,
            base_swap_ratio,
            custom_swap_ratio,
            router,
        } => {
            let config = Config::load(deps.storage)?;
            if !config.is_admin(&info.sender) {
                return Err(ContractError::Unauthorized {});
            }

            crate::executions::update_config(deps, env, info, admin, whitelist, offer_token, base_swap_ratio, custom_swap_ratio, router)
        },
        ExecuteMsg::ApproveAdminNominee {} => crate::executions::approve_admin_nominee(deps, env, info),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Mint {} => {
            let config = Config::load(deps.storage)?;

            if info.sender != config.offer_token {
                return Err(ContractError::Unauthorized {});
            }

            crate::executions::mint(
                deps,
                env,
                info,
                cw20_msg.sender,
                cw20_msg.amount,
            )
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&cw20_base::contract::query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&cw20_base::contract::query_token_info(deps)?),
        QueryMsg::Allowance { owner, spender } => to_binary(&cw20_base::allowances::query_allowance(deps, owner, spender)?),
        QueryMsg::Minter {} => to_binary(&cw20_base::contract::query_minter(deps)?),
        QueryMsg::MarketingInfo {} => to_binary(&cw20_base::contract::query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&cw20_base::contract::query_download_logo(deps)?),
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&cw20_base::enumerable::query_all_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => to_binary(&cw20_base::enumerable::query_all_accounts(deps, start_after, limit)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::SwapState {ratio} => to_binary(&query_swap_state(deps, ratio)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    if cw2::get_contract_version(deps.storage).is_err() {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, "1.0.8-beta.0".to_string())?;
    }

    //mig to v1.0.8-beta.0 to v1.0.8-beta.1
    let info = cw2::get_contract_version(deps.storage)?;
    if info.version == "v1.0.8-beta.0".to_string() {
        let router = &deps.api.addr_validate(msg.router.as_str())?;
        migrations::v108_beta0::migrate(deps.storage, &env, router)?;

        set_contract_version(deps.storage, CONTRACT_NAME, "1.0.8-beta.1")?;
    }

    Ok(Response::default())
}