use crate::campaign::enumerations::{Denom, Referrer};
use cosmwasm_std::{Uint128, Uint64};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CampaignInfo {},
    DistributionConfig {},
    CampaignState {},
    ShareUrl {
        address: String,
    },
    GetAddressFromReferrer {
        referrer: Referrer,
    },
    Participation {
        actor_address: String,
    },
    Participations {},
}

pub struct CampaignInfoResponse {
    pub title: String,
    pub description: String,
    pub url: String,
    pub creator: String,
}

pub struct DistributionConfigResponse {
    pub denom: Denom,
    pub amounts: Vec<Uint128>,
}

pub struct CampaignStateResponse {
    pub participation_count: Uint64,
    pub cumulative_distribution_amount: Uint128,
    pub locked_balance: Uint128,
    pub balance: Uint128,
    pub is_active: bool,
}

pub struct ShareUrlResponse {
    pub address: String,
    pub compressed: String,
    pub url: String,
}

pub struct ParticipationResponse {
    pub actor_address: String,
    pub referrer_address: Option<String>,
    pub rewards: Vec<(Denom, Uint128)>,
    pub share_url: String,
}