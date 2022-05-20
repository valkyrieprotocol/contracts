use cosmwasm_std::MessageInfo;
use cosmwasm_std::testing::mock_info;

pub const DEFAULT_SENDER: &str = "DefaultSender";
pub const CONTRACT_CREATOR: &str = "ContractCreator";
pub const VALKYRIE_TOKEN: &str = "ValkyrieToken";
pub const VALKYRIE_TICKET_TOKEN: &str = "TicketToken";
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
    use crate::test_constants::VALKYRIE_TICKET_TOKEN;
    use crate::test_utils::{mock_env_contract, mock_env_contract_height};

    pub const GOVERNANCE: &str = "Governance";

    // common config
    pub const GOVERNANCE_TOKEN: &str = VALKYRIE_TOKEN;
    pub const TICKET_TOKEN: &str = VALKYRIE_TICKET_TOKEN;
    pub const TICKET_DIST_SCHEDULE: (u64, u64, Uint128) = (0, 100, Uint128::new(100_000000u128));

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

pub mod community {
    use cosmwasm_std::{Env, MessageInfo};
    use cosmwasm_std::testing::mock_info;

    use crate::test_constants::governance::GOVERNANCE;
    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_utils::mock_env_contract;

    pub const COMMUNITY: &str = "Community";

    pub const MANAGING_TOKEN: &str = VALKYRIE_TOKEN;
    pub const ADMIN: &str = GOVERNANCE;
    pub const ALLOWED_ADDRESS: &str = "AllowedAddress";
    // pub const ALLOWED_AMOUNT: Uint128 = Uint128::new(1000);

    pub fn community_env() -> Env {
        mock_env_contract(COMMUNITY)
    }

    pub fn community_sender() -> MessageInfo {
        mock_info(COMMUNITY, &[])
    }
}

pub mod distributor {
    use cosmwasm_std::{Env, MessageInfo};
    use cosmwasm_std::testing::mock_info;

    use crate::test_constants::governance::GOVERNANCE;
    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_utils::mock_env_contract;

    pub const DISTRIBUTOR: &str = "Distributor";

    pub const MANAGING_TOKEN: &str = VALKYRIE_TOKEN;
    pub const ADMIN: &str = GOVERNANCE;

    pub fn distributor_env() -> Env {
        mock_env_contract(DISTRIBUTOR)
    }

    pub fn distributor_sender() -> MessageInfo {
        mock_info(DISTRIBUTOR, &[])
    }
}

pub mod campaign_manager {
    use cosmwasm_std::{Env, MessageInfo};
    use cosmwasm_std::testing::mock_info;

    use crate::test_utils::mock_env_contract;
    use crate::test_constants::governance::GOVERNANCE;

    pub const CAMPAIGN_MANAGER: &str = "CampaignManager";

    pub const CAMPAIGN_CODE_ID: u64 = 1;
    pub const ADD_POOL_FEE_RATE_PERCENT: u64 = 0;
    pub const ADD_POOL_MIN_REFERRAL_REWARD_RATE_PERCENT: u64 = 20;
    pub const REMOVE_POOL_FEE_RATE_PERCENT: u64 = 10;
    pub const FEE_BURN_RATIO_PERCENT: u64 = 50;
    pub const FEE_RECIPIENT: &str = GOVERNANCE;
    pub const CAMPAIGN_DEACTIVATE_PERIOD: u64 = 403290;
    pub const KEY_DENOM_NATIVE: &str = "uluna";
    pub const REFERRAL_REWARD_LIMIT_BASE_COUNT: u8 = 5;
    pub const REFERRAL_REWARD_LIMIT_STAKING_PERCENT: u16 = 50;

    pub fn campaign_manager_env() -> Env {
        mock_env_contract(CAMPAIGN_MANAGER)
    }

    pub fn campaign_manager_sender() -> MessageInfo {
        mock_info(CAMPAIGN_MANAGER, &[])
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
    pub const PARTICIPATION_REWARD_DENOM_NATIVE: &str = "uluna";
    pub const PARTICIPATION_REWARD_AMOUNT: Uint128 = Uint128::new(3000);

    pub const PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1: (u64, u64, Uint128) = (10, 10, Uint128::new(300)); //start, end, amount
    pub const PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2: (u64, u64, Uint128) = (10, 100, Uint128::new(1200));
    pub const PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3: (u64, u64, Uint128) = (100, 200, Uint128::new(1500));

    pub const REFERRAL_REWARD_AMOUNTS: [Uint128; 3] = [Uint128::new(400), Uint128::new(300), Uint128::new(200)];
    pub const REFERRAL_REWARD_LOCK_PERIOD: u64 = 100;
    pub const QUALIFIER: &str = "Qualifier";
    pub const DEPOSIT_DENOM_NATIVE: &str = "uluna";
    pub const DEPOSIT_AMOUNT: Uint128 = Uint128::new(100);
    pub const DEPOSIT_LOCK_PERIOD: u64 = 10000;
    pub const VP_BURN_AMOUNT: Uint128 = Uint128::new(0);

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

pub mod liquidity {
    use cosmwasm_std::{Env, Uint128};
    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_utils::{mock_env_contract, mock_env_contract_height};

    pub const LIQUIDITY: &str = "Liquidity";

    pub const LP_REWARD_TOKEN: &str = VALKYRIE_TOKEN;
    pub const LP_PAIR_TOKEN: &str = "ValkyrieLpPair";
    pub const LP_LIQUIDITY_TOKEN: &str = "ValkyrieLpToken";
    pub const LP_WHITELISTED1: &str = "ValkyrieLpWhitelist1";
    pub const LP_WHITELISTED2: &str = "ValkyrieLpWhitelist2";
    pub const LP_DISTRIBUTION_SCHEDULE1: (u64, u64, Uint128) = (0, 100, Uint128::new(1000000u128));
    pub const LP_DISTRIBUTION_SCHEDULE2: (u64, u64, Uint128) = (100, 200, Uint128::new(10000000u128));

    pub fn lp_env() -> Env {
        let mut env = mock_env_contract(LIQUIDITY);
        env.block.height = 0;
        env
    }

    pub fn governance_env_height(height: u64) -> Env {
        mock_env_contract_height(LIQUIDITY, height)
    }
}