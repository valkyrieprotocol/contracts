use cosmwasm_std::{Addr, Decimal, StdResult, Storage};
use cw20::Denom;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use valkyrie::campaign_manager::query_msgs::{CampaignResponse, CampaignsResponse};
use valkyrie::common::OrderBy;
use valkyrie::pagination::addr_range_option;

const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance: Addr,
    pub fund_manager: Addr,
    pub terraswap_router: Addr,
    pub code_id: u64,
    pub deposit_fee_rate: Decimal,
    pub withdraw_fee_rate: Decimal,
    pub withdraw_fee_recipient: Addr,
    pub deactivate_period: u64,
    pub key_denom: Denom,
    pub referral_reward_token: Addr,
    pub min_referral_reward_deposit_rate: Decimal,
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        CONFIG.load(storage)
    }

    pub fn is_governance(&self, address: &Addr) -> bool {
        self.governance == *address
    }
}


const REFERRAL_REWARD_LIMIT_OPTION: Item<ReferralRewardLimitOption> = Item::new("referral_reward_limit_option");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReferralRewardLimitOption {
    pub overflow_amount_recipient: Option<Addr>,
    pub base_count: u8,
    pub percent_for_governance_staking: u16,
}

impl ReferralRewardLimitOption {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        REFERRAL_REWARD_LIMIT_OPTION.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<ReferralRewardLimitOption> {
        REFERRAL_REWARD_LIMIT_OPTION.load(storage)
    }
}


const CREATE_CAMPAIGN_CONTEXT: Item<CreateCampaignContext> = Item::new("create_campaign_context");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateCampaignContext {
    pub code_id: u64,
    pub creator: Addr,
}

impl CreateCampaignContext {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CREATE_CAMPAIGN_CONTEXT.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<CreateCampaignContext> {
        CREATE_CAMPAIGN_CONTEXT.load(storage)
    }

    #[cfg(test)]
    pub fn may_load(storage: &dyn Storage) -> StdResult<Option<CreateCampaignContext>> {
        CREATE_CAMPAIGN_CONTEXT.may_load(storage)
    }

    pub fn clear(storage: &mut dyn Storage) {
        CREATE_CAMPAIGN_CONTEXT.remove(storage)
    }
}


const CAMPAIGN: Map<&Addr, Campaign> = Map::new("campaign");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Campaign {
    pub code_id: u64,
    pub address: Addr,
    pub creator: Addr,
    pub created_height: u64,
}

impl Campaign {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CAMPAIGN.save(storage, &self.address, self)
    }

    pub fn load(storage: &dyn Storage, address: &Addr) -> StdResult<Campaign> {
        CAMPAIGN.load(storage, address)
    }

    pub fn query(
        storage: &dyn Storage,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<CampaignsResponse> {
        let range_option = addr_range_option(start_after, limit, order_by);

        let campaigns = CAMPAIGN
            .range(storage, range_option.min, range_option.max, range_option.order_by)
            .take(range_option.limit)
            .map(|item| {
                let (_, campaign) = item?;

                Ok(CampaignResponse {
                    code_id: campaign.code_id,
                    address: campaign.address.to_string(),
                    creator: campaign.creator.to_string(),
                    created_height: campaign.created_height,
                })
            })
            .collect::<StdResult<Vec<CampaignResponse>>>()?;

        Ok(CampaignsResponse {
            campaigns,
        })
    }
}