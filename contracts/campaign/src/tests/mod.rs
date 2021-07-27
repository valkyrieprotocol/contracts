use cosmwasm_std::{MessageInfo, Uint128};
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_contract_config;
pub mod update_campaign_info;
pub mod update_distribution_config;
pub mod update_activation;
pub mod withdraw;
pub mod claim_participation_reward;
pub mod claim_booster_reward;
pub mod participate;
pub mod enable_booster;
pub mod disable_booster;

pub const GOVERNANCE: &str = "GovernanceContract";
pub const FUND_MANAGER: &str = "FundManager";
pub const TOKEN_CONTRACT: &str = "TokenContract";
pub const CAMPAIGN_MANAGER: &str = "CampaignManager";
pub const CAMPAIGN_TITLE: &str = "CampaignTitle";
pub const CAMPAIGN_DESCRIPTION: &str = "CamapignDescription";
pub const CAMPAIGN_URL: &str = "https://campaign.url";
pub const CAMPAIGN_PARAMETER_KEY: &str = "vkr";
pub const CAMPAIGN_DISTRIBUTION_DENOM_NATIVE: &str = "uusd";
pub const CAMPAIGN_DISTRIBUTION_AMOUNTS: [Uint128; 3] = [Uint128::new(5), Uint128::new(3), Uint128::new(2)];
pub const CAMPAIGN_ADMIN: &str = "CampaignAdmin";

pub fn campaign_manager_sender() -> MessageInfo {
    mock_info(CAMPAIGN_MANAGER, &[])
}

pub fn campaign_admin_sender() -> MessageInfo {
    mock_info(CAMPAIGN_ADMIN, &[])
}

// pub fn fund_manager_sender() -> MessageInfo {
//     mock_info(FUND_MANAGER, &[])
// }