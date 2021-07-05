use cosmwasm_std::MessageInfo;
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_booster_config;
pub mod add_campaign;
pub mod remove_campaign;
pub mod spend;
pub mod swap;

pub const GOVERNANCE: &str = "GovernanceContract";
pub const TOKEN_CONTRACT: &str = "TokenContract";
pub const TERRASWAP_ROUTER: &str = "TerraswapRouter";
pub const DROP_BOOSTER_RATIO_PERCENT: u64 = 10;
pub const ACTIVITY_BOOSTER_RATIO_PERCENT: u64 = 80;
pub const PLUS_BOOSTER_RATIO_PERCENT: u64 = 10;
pub const ACTIVITY_BOOSTER_MULTIPLIER_PERCENT: u64 = 80;
pub const MIN_PARTICIPATION_COUNT: u64 = 10;

pub fn governance_sender() -> MessageInfo {
    mock_info(GOVERNANCE, &[])
}