use cosmwasm_std::{Deps, Env};

use valkyrie::common::ContractResult;
use valkyrie::governance::query_msgs::{CampaignCodeInfoResponse, ValkyrieConfigResponse};

use super::states::{CampaignCode, ValkyrieConfig};

pub fn get_valkyrie_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<ValkyrieConfigResponse> {
    let valkyrie_config = ValkyrieConfig::load(deps.storage)?;

    Ok(
        ValkyrieConfigResponse {
            campaign_code_whitelist: valkyrie_config.campaign_code_whitelist,
            boost_contract: valkyrie_config.boost_contract.map(|v| v.to_string()),
        }
    )
}

pub fn get_campaign_code_info(
    deps: Deps,
    _env: Env,
    code_id: u64,
) -> ContractResult<CampaignCodeInfoResponse> {
    let campaign_code = CampaignCode::load(deps.storage, &code_id)?;

    Ok(
        CampaignCodeInfoResponse {
            code_id: campaign_code.code_id,
            source_code_url: campaign_code.source_code_url,
            description: campaign_code.description,
            maintainer: campaign_code.maintainer,
        }
    )
}