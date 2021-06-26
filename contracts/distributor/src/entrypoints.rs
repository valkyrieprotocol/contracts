#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::distributor::execute_msgs::{ExecuteMsg, InstantiateMsg};
use valkyrie::distributor::query_msgs::QueryMsg;

use crate::{
    executions::{add_campaign, remove_campaign, spend, update_booster_config},
    queries::{get_campaign_info, get_campaign_infos, get_contract_config},
    states::ContractConfig,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = ContractConfig {
        governance: deps.api.addr_validate(&msg.governance)?,
        token_contract: deps.api.addr_validate(&msg.token_contract)?,
        booster_config: msg.booster_config,
    };

    config.booster_config.validate()?;
    config.save(deps.storage)?;

    Ok(Response::default())
}

//TODO: 받은 token 중 token contract 가 아닌건들은 terraswap을 통해서 token contract 로 전환 필요
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::AddCampaign {
            campaign_addr,
            spend_limit,
        } => add_campaign(deps, env, info, campaign_addr, spend_limit),
        ExecuteMsg::RemoveCampaign { campaign_addr } => {
            remove_campaign(deps, env, info, campaign_addr)
        }
        ExecuteMsg::Spend { recipient, amount } => spend(deps, env, info, recipient, amount),
        ExecuteMsg::UpdateBoosterConfig { booster_config } => {
            update_booster_config(deps, env, info, booster_config)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    let result = match msg {
        QueryMsg::ContractConfig {} => to_binary(&get_contract_config(deps, env)?),
        QueryMsg::CampaignInfo { campaign_addr } => {
            to_binary(&get_campaign_info(deps, env, campaign_addr)?)
        }
        QueryMsg::CampaignInfos {
            start_after,
            limit,
            order_by,
        } => to_binary(&get_campaign_infos(
            deps,
            env,
            start_after,
            limit,
            order_by,
        )?),
    }?;

    Ok(result)
}
