use cosmwasm_std::MessageInfo;
use cosmwasm_std::testing::mock_info;

pub const DEFAULT_SENDER: &str = "terra1sq9ppsvt4k378wwhvm2vyfg7kqrhtve8p0n3a6";
pub const CONTRACT_CREATOR: &str = "terra16m3runusa9csfev7ymj62e8lnswu8um29k5zky";
pub const VALKYRIE_TOKEN: &str = "terra1xj49zyqrwpv5k928jwfpfy2ha668nwdgkwlrg3";
pub const VALKYRIE_TICKET_TOKEN: &str = "terra1fh27l8h4s0tfx9ykqxq5efq4xx88f06x6clwmr";
pub const VALKYRIE_PROXY: &str = "terra1fnywlw4edny3vw44x04xd67uzkdqluymgreu7g";

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

    pub const GOVERNANCE: &str = "terra16t7dpwwgx9n3lq6l6te3753lsjqwhxwpday9zx";

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

    pub const COMMUNITY: &str = "terra1f68wt2ch3cx2g62dxtc8v68mkdh5wchdgdjwz7";

    pub const MANAGING_TOKEN: &str = VALKYRIE_TOKEN;
    pub const ADMIN: &str = GOVERNANCE;

    pub const ADMIN1: &str = "terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f";
    pub const ALLOWED_ADDRESS: &str = "terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f";
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

    pub const DISTRIBUTOR: &str = "terra14lpnyzc9z4g3ugr4lhm8s4nle0tq8vcltkhzh7";

    pub const RECIPIENT: &str = "terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f";
    pub const RECIPIENT2: &str = "terra17q4lzg70un58uefr2fwu7uxtgvftspr7d0a6p3";
    pub const ADMIN1: &str = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";

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

    pub const CAMPAIGN_MANAGER: &str = "terra12u7hcmpltazmmnq0fvyl225usn3fy6qqlp05w0";

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

    pub const CAMPAIGN: &str = "terra1zgrx9jjqrfye8swykfgmd6hpde60j0nszzupp9";
    pub const CAMPAIGN_TITLE: &str = "CampaignTitle";
    pub const CAMPAIGN_DESCRIPTION: &str = "CamapignDescription";
    pub const CAMPAIGN_URL: &str = "https://campaign.url";
    pub const CAMPAIGN_PARAMETER_KEY: &str = "vkr";
    pub const CAMPAIGN_ADMIN: &str = "terra1h8ljdmae7lx05kjj79c9ekscwsyjd3yr8wyvdn";
    pub const PARTICIPATION_REWARD_DENOM_NATIVE: &str = "uluna";
    pub const PARTICIPATION_REWARD_AMOUNT: Uint128 = Uint128::new(3000);

    pub const PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE1: (u64, u64, Uint128) = (10, 10, Uint128::new(300)); //start, end, amount
    pub const PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE2: (u64, u64, Uint128) = (10, 100, Uint128::new(1200));
    pub const PARTICIPATION_REWARD_DISTRIBUTION_SCHEDULE3: (u64, u64, Uint128) = (100, 200, Uint128::new(1500));

    pub const REFERRAL_REWARD_AMOUNTS: [Uint128; 3] = [Uint128::new(400), Uint128::new(300), Uint128::new(200)];
    pub const REFERRAL_REWARD_LOCK_PERIOD: u64 = 100;
    pub const QUALIFIER: &str = "terra1rlusvz7h678g35dr5a3nzmzd2kzwh2evjpfuq8";
    pub const QUALIFIER2: &str = "terra190fxpjfkp6cygr2k9unzjurq42dyehqd579h5j";
    pub const ADMIN2: &str = "terra190fxpjfkp6cygr2k9unzjurq42dyehqd579h5j";
    pub const DEPOSIT_DENOM_NATIVE: &str = "uluna";
    pub const DEPOSIT_AMOUNT: Uint128 = Uint128::new(100);
    pub const DEPOSIT_LOCK_PERIOD: u64 = 10000;
    pub const VP_BURN_AMOUNT: Uint128 = Uint128::new(0);

    pub const REFERRER: &str = "terra1dpe2aqykm2vnakcz4vgpha0agxnlkjvgfahhk7";
    pub const INVALID_REFERRER: &str = "terra174gu7kg8ekk5gsxdma5jlfcedm653tyg6ayppw";
    pub const PARTICIPATOR1: &str = "terra1hk7fturdl9fnvrn566dxer6ds7v4jklp2wqmp7";
    pub const PARTICIPATOR2: &str = "terra1tvld5k6pus2yh7pcu7xuwyjedn7mjxfkkkjjap";
    pub const RECIPIENT: &str = "terra1f68wt2ch3cx2g62dxtc8v68mkdh5wchdgdjwz7";

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

    pub const LIQUIDITY: &str = "terra1l7xu2rl3c7qmtx3r5sd2tz25glf6jh8ul7aag7";

    pub const LP_REWARD_TOKEN: &str = VALKYRIE_TOKEN;
    pub const LP_PAIR_TOKEN: &str = "terra17n5sunn88hpy965mzvt3079fqx3rttnplg779g";
    pub const LP_LIQUIDITY_TOKEN: &str = "terra1627ldjvxatt54ydd3ns6xaxtd68a2vtyu7kakj";
    pub const LP_WHITELISTED1: &str = "terra190fxpjfkp6cygr2k9unzjurq42dyehqd579h5j";
    pub const LP_WHITELISTED2: &str = "terra1c7m6j8ya58a2fkkptn8fgudx8sqjqvc8azq0ex";
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

pub mod proxy {
    use cosmwasm_std::{Env, MessageInfo, Uint128};
    use cosmwasm_std::testing::mock_info;

    use crate::test_constants::VALKYRIE_TOKEN;
    use crate::test_constants::VALKYRIE_TICKET_TOKEN;
    use crate::test_utils::{mock_env_contract, mock_env_contract_height};
    pub const PROXY: &str = "terra190fxpjfkp6cygr2k9unzjurq42dyehqd579h5j";
    pub const ASTRO_FACTORY: &str = "terra16t7dpwwgx9n3lq6l6te3753lsjqwhxwpday9zx";
    pub const ADMIN: &str = "terra1hk7fturdl9fnvrn566dxer6ds7v4jklp2wqmp7";
    pub const ADMIN2: &str = "terra1c7m6j8ya58a2fkkptn8fgudx8sqjqvc8azq0ex";

    pub fn proxy_env() -> Env {
        mock_env_contract(PROXY)
    }

    pub fn proxy_env_height(height: u64) -> Env {
        mock_env_contract_height(PROXY, height)
    }

    pub fn proxy_sender(sender:&str) -> MessageInfo {
        mock_info(sender, &[])
    }


}