use cosmwasm_std::{MessageInfo, Uint128};
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_campaign_info;
pub mod update_distribution_config;
pub mod update_admin;
pub mod update_activation;
pub mod withdraw_reward;
pub mod claim_reward;
pub mod participate;
pub mod register_booster;
pub mod deregister_booster;

pub const GOVERNANCE: &str = "GovernanceContract";
pub const DISTRIBUTOR: &str = "DistributorContract";
pub const TOKEN_CONTRACT: &str = "TokenContract";
pub const FACTORY: &str = "FactoryContract";
pub const BURN_CONTRACT: &str = "BurnContract";
pub const CAMPAIGN_TITLE: &str = "CampaignTitle";
pub const CAMPAIGN_DESCRIPTION: &str = "CamapignDescription";
pub const CAMPAIGN_URL: &str = "https://campaign.url";
pub const CAMPAIGN_PARAMETER_KEY: &str = "vkr";
pub const CAMPAIGN_DISTRIBUTION_DENOM_NATIVE: &str = "uusd";
pub const CAMPAIGN_DISTRIBUTION_AMOUNTS: [Uint128; 3] = [Uint128(10), Uint128(8), Uint128(2)];
pub const CAMPAIGN_ADMIN: &str = "CampaignAdmin";

pub fn factory_sender() -> MessageInfo {
    mock_info(FACTORY, &[])
}

pub fn campaign_admin_sender() -> MessageInfo {
    mock_info(CAMPAIGN_ADMIN, &[])
}

pub fn distributor_sender() -> MessageInfo {
    mock_info(DISTRIBUTOR, &[])
}