use cosmwasm_std::{Deps, Env, Uint128, Uint64};

use valkyrie::common::ContractResult;
use valkyrie::factory::query_msgs::{CampaignResponse, ConfigResponse};

use crate::states::{Campaign, FactoryConfig};

pub fn get_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<ConfigResponse> {
    let factory_config = FactoryConfig::load(deps.storage)?;

    Ok(ConfigResponse {
        governance: factory_config.governance.to_string(),
        token_contract: factory_config.token_contract.to_string(),
        campaign_code_id: Uint64::from(factory_config.campaign_code_id),
        creation_fee_amount: Uint128::from(factory_config.creation_fee_amount),
    })
}

pub fn get_campaign(
    deps: Deps,
    _env: Env,
    address: String,
) -> ContractResult<CampaignResponse> {
    let campaign = Campaign::load(
        deps.storage,
        &deps.api.addr_validate(address.as_str())?,
    )?;

    Ok(CampaignResponse {
        code_id: Uint64::from(campaign.code_id),
        address: campaign.address.to_string(),
        creator: campaign.creator.to_string(),
        created_block: Uint64::from(campaign.created_block),
    })
}