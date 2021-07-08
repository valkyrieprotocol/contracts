use cosmwasm_std::{Uint128, MessageInfo};
use cosmwasm_std::testing::mock_info;

pub mod instantiate;
pub mod update_contract_config;
pub mod update_campaign_config;
pub mod update_booster_config;
pub mod add_distribution_denom;
pub mod remove_distribution_denom;
pub mod create_campaign;
pub mod created_campaign;
pub mod boost_campaign;
pub mod finish_boosting;


pub const GOVERNANCE: &str = "GovernanceContract";
pub const FUND_MANAGER: &str = "FundManager";
pub const TOKEN_CONTRACT: &str = "TokenContract";
pub const CREATION_FEE_AMOUNT: Uint128 = Uint128(100000000);
pub const CAMPAIGN_CODE_ID: u64 = 1;
pub const DISTRIBUTION_DENOM_WHITELIST_NATIVE: &str = "uusd";
pub const DISTRIBUTION_DENOM_WHITELIST_TOKEN: &str = TOKEN_CONTRACT;
pub const WITHDRAW_FEE_RATE_PERCENT: u64 = 10;
pub const CAMPAIGN_DEACTIVATE_PERIOD: u64 = 403290;
pub const DROP_BOOSTER_RATIO_PERCENT: u64 = 10;
pub const ACTIVITY_BOOSTER_RATIO_PERCENT: u64 = 80;
pub const PLUS_BOOSTER_RATIO_PERCENT: u64 = 10;
pub const ACTIVITY_BOOSTER_MULTIPLIER_PERCENT: u64 = 80;
pub const MIN_PARTICIPATION_COUNT: u64 = 10;


pub fn governance_sender() -> MessageInfo {
    mock_info(GOVERNANCE, &[])
}