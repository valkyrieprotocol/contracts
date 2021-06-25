use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Decimal};

use valkyrie::common::ContractResult;
use valkyrie::errors::ContractError;

use crate::common::states::is_admin;

use super::states::ValkyrieConfig;
use valkyrie::governance::execute_msgs::ValkyrieConfigInitMsg;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ValkyrieConfigInitMsg,
) -> ContractResult<Response> {
    // Execute
    let config = ValkyrieConfig {
        burn_contract: deps.api.addr_validate(msg.burn_contract.as_str())?,
        reward_withdraw_burn_rate: msg.reward_withdraw_burn_rate,
    };

    config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    burn_contract: Option<String>,
    reward_withdraw_burn_rate: Option<Decimal>,
) -> ContractResult<Response> {
    // Validate
    if !is_admin(deps.storage, env, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Execute
    let mut valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    if let Some(burn_contract) = burn_contract {
        valkyrie_config.burn_contract = deps.api.addr_validate(&burn_contract)?;
    }

    if let Some(reward_withdraw_burn_rate) = reward_withdraw_burn_rate {
        valkyrie_config.reward_withdraw_burn_rate = reward_withdraw_burn_rate;
    }

    valkyrie_config.save(deps.storage)?;

    // Response
    Ok(Response::default())
}
