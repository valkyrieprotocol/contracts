use cosmwasm_std::{Addr, Decimal, Env, MessageInfo, Response, Uint128};

use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::campaign_manager_env;
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_utils::expect_unauthorized_err;

use crate::executions::update_config;
use crate::states::Config;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: Option<String>,
    fund_manager: Option<String>,
    terraswap_router: Option<String>,
    creation_fee_token: Option<String>,
    creation_fee_amount: Option<Uint128>,
    creation_fee_recipient: Option<String>,
    code_id: Option<u64>,
    deposit_fee_rate: Option<Decimal>,
    withdraw_fee_rate: Option<Decimal>,
    withdraw_fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
    key_denom: Option<Denom>,
    referral_reward_token: Option<String>,
    min_referral_reward_deposit_rate: Option<Decimal>,
) -> ContractResult<Response> {
    update_config(
        deps.as_mut(),
        env,
        info,
        governance,
        fund_manager,
        terraswap_router,
        creation_fee_token,
        creation_fee_amount,
        creation_fee_recipient,
        code_id,
        deposit_fee_rate,
        withdraw_fee_rate,
        withdraw_fee_recipient,
        deactivate_period,
        key_denom,
        referral_reward_token,
        min_referral_reward_deposit_rate,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    governance: Option<String>,
    fund_manager: Option<String>,
    terraswap_router: Option<String>,
    creation_fee_token: Option<String>,
    creation_fee_amount: Option<Uint128>,
    creation_fee_recipient: Option<String>,
    code_id: Option<u64>,
    deposit_fee_rate: Option<Decimal>,
    withdraw_fee_rate: Option<Decimal>,
    withdraw_fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
    key_denom: Option<Denom>,
    referral_reward_token: Option<String>,
    min_referral_reward_deposit_rate: Option<Decimal>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        governance,
        fund_manager,
        terraswap_router,
        creation_fee_token,
        creation_fee_amount,
        creation_fee_recipient,
        code_id,
        deposit_fee_rate,
        withdraw_fee_rate,
        withdraw_fee_recipient,
        deactivate_period,
        key_denom,
        referral_reward_token,
        min_referral_reward_deposit_rate,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let governance = "ChangedGovernance";
    let fund_manager = "ChangedFundManager";
    let terraswap_router = "ChangedTerraswapRouter";
    let creation_fee_token = "ChangedFeeToken";
    let creation_fee_amount = Uint128::new(1);
    let creation_fee_recipient = "ChangedFeeRecipient";
    let code_id = 100u64;
    let deposit_fee_rate = Decimal::percent(9);
    let withdraw_fee_rate = Decimal::percent(99);
    let withdraw_fee_recipient = "ChangedFeeRecipient";
    let deactivate_period = 99u64;
    let key_denom = Denom::Native("ukrw".to_string());
    let referral_reward_token = "ChangedRefRewardToken";
    let min_referral_reward_deposit_rate = Decimal::percent(20);

    will_success(
        &mut deps,
        Some(governance.to_string()),
        Some(fund_manager.to_string()),
        Some(terraswap_router.to_string()),
        Some(creation_fee_token.to_string()),
        Some(creation_fee_amount.clone()),
        Some(creation_fee_recipient.to_string()),
        Some(code_id),
        Some(deposit_fee_rate),
        Some(withdraw_fee_rate),
        Some(withdraw_fee_recipient.to_string()),
        Some(deactivate_period),
        Some(key_denom.clone()),
        Some(referral_reward_token.to_string()),
        Some(min_referral_reward_deposit_rate),
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config, Config {
        governance: Addr::unchecked(governance),
        fund_manager: Addr::unchecked(fund_manager),
        terraswap_router: Addr::unchecked(terraswap_router),
        creation_fee_token: Addr::unchecked(creation_fee_token),
        creation_fee_amount: creation_fee_amount.clone(),
        creation_fee_recipient: Addr::unchecked(creation_fee_recipient),
        code_id: code_id.clone(),
        deposit_fee_rate: deposit_fee_rate.clone(),
        withdraw_fee_rate: withdraw_fee_rate.clone(),
        withdraw_fee_recipient: Addr::unchecked(withdraw_fee_recipient),
        deactivate_period: deactivate_period.clone(),
        key_denom: key_denom.to_cw20(&deps.api),
        referral_reward_token: Addr::unchecked(referral_reward_token),
        min_referral_reward_deposit_rate: min_referral_reward_deposit_rate.clone(),
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        campaign_manager_env(),
        default_sender(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    expect_unauthorized_err(&result);
}
