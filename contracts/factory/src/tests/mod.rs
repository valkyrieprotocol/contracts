use cosmwasm_std::{Uint128, MessageInfo};
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_factory_config;
pub mod update_campaign_config;
pub mod create_campaign;
pub mod reply_created_campaign;


pub const GOVERNANCE: &str = "GovernanceContract";
pub const TOKEN_CONTRACT: &str = "TokenContract";
pub const DISTRIBUTOR: &str = "DistributorContract";
pub const BURN_CONTRACT: &str = "BurnContract";
pub const CAMPAIGN_CODE_ID: u64 = 1;
pub const CREATION_FEE_AMOUNT: Uint128 = Uint128(100000000);
pub const REWARD_WITHDRAW_BURN_RATE_PERCENT: u64 = 10;
pub const CAMPAIGN_DEACTIVATE_PERIOD: u64 = 403290;

pub fn governance_sender() -> MessageInfo {
    mock_info(GOVERNANCE, &[])
}