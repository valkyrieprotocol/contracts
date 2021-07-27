use cosmwasm_std::MessageInfo;
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_config;
pub mod increase_allowance;
pub mod decrease_allowance;
pub mod transfer;
pub mod swap;

pub const GOVERNANCE: &str = "Governance";
pub const CAMPAIGN_MANAGER: &str = "CampaignManager";
pub const ADMINS: [&str; 2] = [GOVERNANCE, CAMPAIGN_MANAGER];
pub const TOKEN_CONTRACT: &str = "TokenContract";
pub const TERRASWAP_ROUTER: &str = "TerraswapRouter";
pub const ALLOWED_ADDRESS: &str = "AllowedAddress";
// pub const ALLOWED_AMOUNT: Uint128 = Uint128::new(1000);


pub fn governance_sender() -> MessageInfo {
    mock_info(GOVERNANCE, &[])
}

pub fn campaign_manager_sender() -> MessageInfo {
    mock_info(CAMPAIGN_MANAGER, &[])
}