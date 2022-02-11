use cosmwasm_std::{Addr, Decimal, Env, MessageInfo, Response};

use valkyrie::campaign_manager::execute_msgs::{InstantiateMsg, ReferralRewardLimitOptionMsg};
use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::*;
use valkyrie::test_constants::{default_sender, TERRASWAP_ROUTER, VALKYRIE_TICKET_TOKEN, VALKYRIE_TOKEN};
use valkyrie::test_constants::governance::GOVERNANCE;

use crate::executions::instantiate;
use crate::states::{Config, ReferralRewardLimitOption};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: String,
    valkyrie_token: String,
    terraswap_router: String,
    code_id: u64,
    add_pool_fee_rate: Decimal,
    add_pool_min_referral_reward_rate: Decimal,
    remove_pool_fee_rate: Decimal,
    fee_burn_ratio: Decimal,
    fee_recipient: String,
    deactivate_period: u64,
    key_denom: Denom,
    overflow_amount_recipient: Option<String>,
    base_count: u8,
    percent_for_governance_staking: u16,
    contract_admin: String,
    vp_token: String,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        governance,
        valkyrie_token,
        terraswap_router,
        code_id,
        add_pool_fee_rate,
        add_pool_min_referral_reward_rate,
        remove_pool_fee_rate,
        fee_burn_ratio,
        fee_recipient,
        deactivate_period,
        key_denom,
        referral_reward_limit_option: ReferralRewardLimitOptionMsg {
            overflow_amount_recipient,
            base_count,
            percent_for_governance_staking,
        },
        contract_admin,
        vp_token,
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
        VALKYRIE_TOKEN.to_string(),
        TERRASWAP_ROUTER.to_string(),
        CAMPAIGN_CODE_ID,
        Decimal::percent(ADD_POOL_FEE_RATE_PERCENT),
        Decimal::percent(ADD_POOL_MIN_REFERRAL_REWARD_RATE_PERCENT),
        Decimal::percent(REMOVE_POOL_FEE_RATE_PERCENT),
        Decimal::percent(FEE_BURN_RATIO_PERCENT),
        FEE_RECIPIENT.to_string(),
        CAMPAIGN_DEACTIVATE_PERIOD,
        Denom::Native(KEY_DENOM_NATIVE.to_string()),
        None,
        REFERRAL_REWARD_LIMIT_BASE_COUNT,
        REFERRAL_REWARD_LIMIT_STAKING_PERCENT,
        GOVERNANCE.to_string(),
        VALKYRIE_TICKET_TOKEN.to_string(),
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
        valkyrie_token: Addr::unchecked(VALKYRIE_TOKEN),
        terraswap_router: Addr::unchecked(TERRASWAP_ROUTER),
        code_id: CAMPAIGN_CODE_ID,
        add_pool_fee_rate: Decimal::percent(ADD_POOL_FEE_RATE_PERCENT),
        add_pool_min_referral_reward_rate: Decimal::percent(ADD_POOL_MIN_REFERRAL_REWARD_RATE_PERCENT),
        remove_pool_fee_rate: Decimal::percent(REMOVE_POOL_FEE_RATE_PERCENT),
        fee_burn_ratio: Decimal::percent(FEE_BURN_RATIO_PERCENT),
        fee_recipient: Addr::unchecked(FEE_RECIPIENT),
        deactivate_period: CAMPAIGN_DEACTIVATE_PERIOD,
        key_denom: cw20::Denom::Native(KEY_DENOM_NATIVE.to_string()),
        contract_admin: Addr::unchecked(GOVERNANCE),
        vp_token: Addr::unchecked(VALKYRIE_TICKET_TOKEN),
    });

    let referral_reward_limit_option = ReferralRewardLimitOption::load(&deps.storage).unwrap();
    assert_eq!(referral_reward_limit_option, ReferralRewardLimitOption {
        overflow_amount_recipient: None,
        base_count: REFERRAL_REWARD_LIMIT_BASE_COUNT,
        percent_for_governance_staking: REFERRAL_REWARD_LIMIT_STAKING_PERCENT,
    });
}
