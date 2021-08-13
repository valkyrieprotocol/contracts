use cosmwasm_std::{Addr, Decimal, Env, MessageInfo, Response};

use valkyrie::campaign_manager::execute_msgs::{InstantiateMsg, ReferralRewardLimitOptionMsg};
use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::*;
use valkyrie::test_constants::{default_sender, TERRASWAP_ROUTER};
use valkyrie::test_constants::fund_manager::FUND_MANAGER;
use valkyrie::test_constants::governance::GOVERNANCE;

use crate::executions::instantiate;
use crate::states::{Config, ReferralRewardLimitOption};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: String,
    fund_manager: String,
    terraswap_router: String,
    code_id: u64,
    deposit_fee_rate: Decimal,
    withdraw_fee_rate: Decimal,
    withdraw_fee_recipient: String,
    deactivate_period: u64,
    key_denom: Denom,
    referral_reward_token: String,
    min_referral_reward_deposit_rate: Decimal,
    overflow_amount_recipient: Option<String>,
    base_count: u8,
    percent_for_governance_staking: u16,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        governance,
        fund_manager,
        terraswap_router,
        code_id,
        deposit_fee_rate,
        withdraw_fee_rate,
        withdraw_fee_recipient,
        deactivate_period,
        key_denom,
        referral_reward_token,
        min_referral_reward_deposit_rate,
        referral_reward_limit_option: ReferralRewardLimitOptionMsg {
            overflow_amount_recipient,
            base_count,
            percent_for_governance_staking,
        }
    };

    instantiate(
        deps.as_mut(),
        env,
        info,
        msg,
    )
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        GOVERNANCE.to_string(),
        FUND_MANAGER.to_string(),
        TERRASWAP_ROUTER.to_string(),
        CAMPAIGN_CODE_ID,
        Decimal::percent(DEPOSIT_FEE_RATE_PERCENT),
        Decimal::percent(WITHDRAW_FEE_RATE_PERCENT),
        FUND_MANAGER.to_string(),
        CAMPAIGN_DEACTIVATE_PERIOD,
        Denom::Native(KEY_DENOM_NATIVE.to_string()),
        REFERRAL_REWARD_TOKEN.to_string(),
        Decimal::percent(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT),
        None,
        REFERRAL_REWARD_LIMIT_BASE_COUNT,
        REFERRAL_REWARD_LIMIT_STAKING_PERCENT,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config, Config {
        governance: Addr::unchecked(GOVERNANCE),
        fund_manager: Addr::unchecked(FUND_MANAGER),
        terraswap_router: Addr::unchecked(TERRASWAP_ROUTER),
        code_id: CAMPAIGN_CODE_ID,
        deposit_fee_rate: Decimal::percent(DEPOSIT_FEE_RATE_PERCENT),
        withdraw_fee_rate: Decimal::percent(WITHDRAW_FEE_RATE_PERCENT),
        withdraw_fee_recipient: Addr::unchecked(FUND_MANAGER),
        deactivate_period: CAMPAIGN_DEACTIVATE_PERIOD,
        key_denom: cw20::Denom::Native(KEY_DENOM_NATIVE.to_string()),
        referral_reward_token: Addr::unchecked(REFERRAL_REWARD_TOKEN),
        min_referral_reward_deposit_rate: Decimal::percent(MIN_REFERRAL_REWARD_DEPOSIT_RATE_PERCENT),
    });

    let referral_reward_limit_option = ReferralRewardLimitOption::load(&deps.storage).unwrap();
    assert_eq!(referral_reward_limit_option, ReferralRewardLimitOption {
        overflow_amount_recipient: None,
        base_count: REFERRAL_REWARD_LIMIT_BASE_COUNT,
        percent_for_governance_staking: REFERRAL_REWARD_LIMIT_STAKING_PERCENT,
    });
}
