use cosmwasm_std::{Deps, Env, Uint128};

use valkyrie::common::ContractResult;
use valkyrie::factory::query_msgs::{CampaignResponse, FactoryConfigResponse, CampaignConfigResponse};

use crate::states::{Campaign, FactoryConfig, CampaignConfig};

pub fn get_factory_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<FactoryConfigResponse> {
    let factory_config = FactoryConfig::load(deps.storage)?;

    Ok(FactoryConfigResponse {
        governance: factory_config.governance.to_string(),
        token_contract: factory_config.token_contract.to_string(),
        distributor: factory_config.distributor.to_string(),
        campaign_code_id: factory_config.campaign_code_id,
        creation_fee_amount: Uint128::from(factory_config.creation_fee_amount),
    })
}

pub fn get_campaign_config(
    deps: Deps,
    _env: Env,
) -> ContractResult<CampaignConfigResponse> {
    let campaign_config = CampaignConfig::load(deps.storage)?;

    Ok(CampaignConfigResponse {
        reward_withdraw_burn_rate: campaign_config.reward_withdraw_burn_rate,
        campaign_deactivate_period: campaign_config.campaign_deactivate_period,
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
        code_id: campaign.code_id,
        address: campaign.address.to_string(),
        creator: campaign.creator.to_string(),
        created_block: campaign.created_block,
    })
}