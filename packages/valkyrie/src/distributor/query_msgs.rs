use crate::common::OrderBy;
use crate::distributor::execute_msgs::BoosterConfig;

use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ContractConfig {},
    CampaignInfo {
        campaign_addr: String,
    },
    CampaignInfos {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractConfigResponse {
    pub governance: String,
    pub token_contract: String,
    pub booster_config: BoosterConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInfoResponse {
    pub campaign_addr: String,
    pub spend_limit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignInfosResponse {
    pub campaigns: Vec<CampaignInfoResponse>,
}
