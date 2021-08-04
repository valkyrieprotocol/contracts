use cosmwasm_std::MessageInfo;
use cosmwasm_std::testing::mock_info;

pub const DEFAULT_SENDER: &str = "DefaultSender";
pub const CONTRACT_CREATOR: &str = "ContractCreator";
pub const VALKYRIE_TOKEN: &str = "ValkyrieToken";
pub const TERRASWAP_ROUTER: &str = "TerraswapRouter";

pub fn default_sender() -> MessageInfo {
    mock_info(DEFAULT_SENDER, &[])
}

pub fn contract_creator() -> MessageInfo {
    mock_info(CONTRACT_CREATOR, &[])
}

pub fn valkyrie_token() -> MessageInfo {
    mock_info(VALKYRIE_TOKEN, &[])
}

pub mod governance {
    use cosmwasm_std::{Env, MessageInfo, Uint128};
    use cosmwasm_std::testing::mock_info;

    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_utils::{mock_env_contract, mock_env_contract_height};

    pub const GOVERNANCE: &str = "Governance";

    // common config
    pub const GOVERNANCE_TOKEN: &str = VALKYRIE_TOKEN;

    // staking config
    pub const WITHDRAW_DELAY: u64 = 94097;

    // poll config
    pub const POLL_QUORUM_PERCENT: u64 = 30;
    pub const POLL_THRESHOLD_PERCENT: u64 = 50;
    pub const POLL_VOTING_PERIOD: u64 = 10000u64;
    pub const POLL_EXECUTION_DELAY_PERIOD: u64 = 10000u64;
    pub const POLL_PROPOSAL_DEPOSIT: Uint128 = Uint128::new(10000000000u128);
    pub const POLL_SNAPSHOT_PERIOD: u64 = 10u64;

    pub fn governance_env() -> Env {
        mock_env_contract(GOVERNANCE)
    }

    pub fn governance_env_height(height: u64) -> Env {
        mock_env_contract_height(GOVERNANCE, height)
    }

    pub fn governance_sender() -> MessageInfo {
        mock_info(GOVERNANCE, &[])
    }
}

pub mod fund_manager {
    use cosmwasm_std::{Env, MessageInfo};
    use cosmwasm_std::testing::mock_info;

    use crate::test_constants::campaign_manager::CAMPAIGN_MANAGER;
    use crate::test_constants::governance::GOVERNANCE;
    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_utils::mock_env_contract;

    pub const FUND_MANAGER: &str = "FundManager";

    pub const MANAGING_TOKEN: &str = VALKYRIE_TOKEN;
    pub const ADMINS: [&str; 2] = [GOVERNANCE, CAMPAIGN_MANAGER];
    pub const ALLOWED_ADDRESS: &str = "AllowedAddress";
    // pub const ALLOWED_AMOUNT: Uint128 = Uint128::new(1000);

    pub fn fund_manager_env() -> Env {
        mock_env_contract(FUND_MANAGER)
    }

    pub fn fund_manager_sender() -> MessageInfo {
        mock_info(FUND_MANAGER, &[])
    }
}

pub mod campaign_manager {
    use cosmwasm_std::{Env, MessageInfo, Uint128};
    use cosmwasm_std::testing::mock_info;

    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_utils::mock_env_contract;

    pub const CAMPAIGN_MANAGER: &str = "CampaignManager";

    pub const CREATION_FEE_TOKEN: &str = VALKYRIE_TOKEN;
    pub const CREATION_FEE_AMOUNT: Uint128 = Uint128::new(100000000);
    pub const CAMPAIGN_CODE_ID: u64 = 1;
    pub const WITHDRAW_FEE_RATE_PERCENT: u64 = 10;
    pub const CAMPAIGN_DEACTIVATE_PERIOD: u64 = 403290;
    pub const KEY_DENOM_NATIVE: &str = "uusd";
    pub const REFERRAL_REWARD_TOKEN: &str = VALKYRIE_TOKEN;
    pub const MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT: u64 = 20;

    pub fn campaign_manager_env() -> Env {
        mock_env_contract(CAMPAIGN_MANAGER)
    }

    pub fn campaign_manager_sender() -> MessageInfo {
        mock_info(CAMPAIGN_MANAGER, &[])
    }

    pub fn creation_fee_token() -> MessageInfo {
        mock_info(CREATION_FEE_TOKEN, &[])
    }
}

pub mod campaign {
    use cosmwasm_std::{Env, MessageInfo, Uint128};
    use cosmwasm_std::testing::mock_info;

    use crate::test_utils::{mock_env_contract, mock_env_contract_height};

    pub const CAMPAIGN: &str = "Campaign";
    pub const CAMPAIGN_TITLE: &str = "CampaignTitle";
    pub const CAMPAIGN_DESCRIPTION: &str = "CamapignDescription";
    pub const CAMPAIGN_URL: &str = "https://campaign.url";
    pub const CAMPAIGN_PARAMETER_KEY: &str = "vkr";
    pub const CAMPAIGN_ADMIN: &str = "CampaignAdmin";
    pub const PARTICIPATION_REWARD_DENOM_NATIVE: &str = "uusd";
    pub const PARTICIPATION_REWARD_AMOUNT: Uint128 = Uint128::new(5);
    pub const REFERRAL_REWARD_AMOUNTS: [Uint128; 3] = [Uint128::new(5), Uint128::new(3), Uint128::new(2)];
    pub const TICKET_AMOUNT: u64 = 1;
    pub const QUALIFIER: &str = "Qualifier";

    pub fn campaign_env() -> Env {
        mock_env_contract(CAMPAIGN)
    }

    pub fn campaign_env_height(height: u64) -> Env {
        mock_env_contract_height(CAMPAIGN, height)
    }

    pub fn campaign_sender() -> MessageInfo {
        mock_info(CAMPAIGN, &[])
    }

    pub fn campaign_admin_sender() -> MessageInfo {
        mock_info(CAMPAIGN_ADMIN, &[])
    }
}
