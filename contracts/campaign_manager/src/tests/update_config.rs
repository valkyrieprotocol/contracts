use cosmwasm_std::{Addr, Decimal, Env, MessageInfo, Response};

use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::campaign_manager_env;
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::{GOVERNANCE, governance_sender};
use valkyrie::test_utils::expect_unauthorized_err;

use crate::executions::update_config;
use crate::states::Config;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: Option<String>,
    valkyrie_token: Option<String>,
    vp_token: Option<String>,
    valkyrie_proxy: Option<String>,
    code_id: Option<u64>,
    add_pool_fee_rate: Option<Decimal>,
    add_pool_min_referral_reward_rate: Option<Decimal>,
    remove_pool_fee_rate: Option<Decimal>,
    fee_burn_ratio: Option<Decimal>,
    fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
    key_denom: Option<Denom>,
    contract_admin: Option<String>,
) -> ContractResult<Response> {
    update_config(
        deps.as_mut(),
        env,
        info,
        governance,
        valkyrie_token,
        vp_token,
        valkyrie_proxy,
        code_id,
        add_pool_fee_rate,
        add_pool_min_referral_reward_rate,
        remove_pool_fee_rate,
        fee_burn_ratio,
        fee_recipient,
        deactivate_period,
        key_denom,
        contract_admin,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    governance: Option<String>,
    valkyrie_token: Option<String>,
    vp_token: Option<String>,
    valkyrie_proxy: Option<String>,
    code_id: Option<u64>,
    add_pool_fee_rate: Option<Decimal>,
    add_pool_min_referral_reward_rate: Option<Decimal>,
    remove_pool_fee_rate: Option<Decimal>,
    fee_burn_ratio: Option<Decimal>,
    fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
    key_denom: Option<Denom>,
    contract_admin: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = campaign_manager_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        governance,
        valkyrie_token,
        vp_token,
        valkyrie_proxy,
        code_id,
        add_pool_fee_rate,
        add_pool_min_referral_reward_rate,
        remove_pool_fee_rate,
        fee_burn_ratio,
        fee_recipient,
        deactivate_period,
        key_denom,
        contract_admin,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let governance = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";
    let valkyrie_token = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";
    let vp_token = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";
    let valkyrie_proxy = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";
    let code_id = 100u64;
    let add_pool_fee_rate = Decimal::percent(9);
    let add_pool_min_referral_reward_rate = Decimal::percent(20);
    let remove_pool_fee_rate = Decimal::percent(99);
    let fee_burn_ratio = Decimal::percent(90);
    let fee_recipient = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";
    let deactivate_period = 99u64;
    let key_denom = Denom::Native("ukrw".to_string());
    let contract_admin = "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n";

    will_success(
        &mut deps,
        Some(governance.to_string()),
        Some(valkyrie_token.to_string()),
        Some(vp_token.to_string()),
        Some(valkyrie_proxy.to_string()),
        Some(code_id),
        Some(add_pool_fee_rate),
        Some(add_pool_min_referral_reward_rate),
        Some(remove_pool_fee_rate),
        Some(fee_burn_ratio),
        Some(fee_recipient.to_string()),
        Some(deactivate_period),
        Some(key_denom.clone()),
        Some(contract_admin.to_string()),
    );

    let config = Config::load(&deps.storage).unwrap();
    assert_eq!(config, Config {
        governance: Addr::unchecked(governance),
        vp_token: Addr::unchecked(vp_token),
        valkyrie_token: Addr::unchecked(valkyrie_token),
        valkyrie_proxy: Addr::unchecked(valkyrie_proxy),
        code_id: code_id.clone(),
        add_pool_fee_rate: add_pool_fee_rate.clone(),
        add_pool_min_referral_reward_rate: add_pool_min_referral_reward_rate.clone(),
        remove_pool_fee_rate: remove_pool_fee_rate.clone(),
        fee_burn_ratio: fee_burn_ratio.clone(),
        fee_recipient: Addr::unchecked(fee_recipient),
        deactivate_period: deactivate_period.clone(),
        key_denom: key_denom.to_cw20(&deps.api),
        contract_admin: Addr::unchecked(GOVERNANCE),
    });

    let admin_nominee = Config::may_load_contract_admin_nominee(&deps.storage).unwrap();
    assert_eq!(admin_nominee, Some(Addr::unchecked(contract_admin)));
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
    );
    expect_unauthorized_err(&result);
}
