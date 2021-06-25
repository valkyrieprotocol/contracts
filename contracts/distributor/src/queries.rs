use cosmwasm_std::{Deps, Env};

use valkyrie::common::{ContractResult, OrderBy};
use valkyrie::distributor::query_msgs::{
    CampaignInfoResponse, CampaignInfosResponse, ContractConfigResponse,
};

use crate::states::{CampaignInfo, ContractConfig};

pub fn get_contract_config(deps: Deps, _env: Env) -> ContractResult<ContractConfigResponse> {
    let contract_config: ContractConfig = ContractConfig::load(deps.storage)?;
    Ok(ContractConfigResponse {
        governance: contract_config.governance.to_string(),
        token_contract: contract_config.token_contract.to_string(),
        booster_config: contract_config.booster_config,
    })
}

pub fn get_campaign_info(
    deps: Deps,
    _env: Env,
    campaign: String,
) -> ContractResult<CampaignInfoResponse> {
    let campaign_info: CampaignInfo =
        CampaignInfo::load(deps.storage, &deps.api.addr_validate(&campaign)?)?;
    Ok(CampaignInfoResponse {
        campaign_addr: campaign_info.campaign.to_string(),
        spend_limit: campaign_info.spend_limit,
    })
}

pub fn get_campaign_infos(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> ContractResult<CampaignInfosResponse> {
    Ok(CampaignInfo::query(
        deps.storage,
        start_after,
        limit,
        order_by,
    )?)
}
