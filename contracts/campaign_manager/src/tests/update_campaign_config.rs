use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, Uint128, Decimal};
use valkyrie::common::ContractResult;
use crate::executions::update_campaign_config;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::governance_sender;
use crate::states::CampaignConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    creation_fee_token: Option<String>,
    creation_fee_amount: Option<Uint128>,
    creation_fee_recipient: Option<String>,
    code_id: Option<u64>,
    withdraw_fee_rate: Option<Decimal>,
    withdraw_fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
) -> ContractResult<Response> {
    update_campaign_config(
        deps.as_mut(),
        env,
        info,
        creation_fee_token,
        creation_fee_amount,
        creation_fee_recipient,
        code_id,
        withdraw_fee_rate,
        withdraw_fee_recipient,
        deactivate_period,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    creation_fee_token: Option<String>,
    creation_fee_amount: Option<Uint128>,
    creation_fee_recipient: Option<String>,
    code_id: Option<u64>,
    withdraw_fee_rate: Option<Decimal>,
    withdraw_fee_recipient: Option<String>,
    deactivate_period: Option<u64>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        creation_fee_token,
        creation_fee_amount,
        creation_fee_recipient,
        code_id,
        withdraw_fee_rate,
        withdraw_fee_recipient,
        deactivate_period,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let creation_fee_token = "ChangedFeeToken";
    let creation_fee_amount = Uint128(1);
    let creation_fee_recipient = "ChangedFeeRecipient";
    let code_id = 100u64;
    let withdraw_fee_rate = Decimal::percent(99);
    let withdraw_fee_recipient = "ChangedFeeRecipient";
    let deactivate_period = 99u64;

    will_success(
        &mut deps,
        Some(creation_fee_token.to_string()),
        Some(creation_fee_amount.clone()),
        Some(creation_fee_recipient.to_string()),
        Some(code_id),
        Some(withdraw_fee_rate),
        Some(withdraw_fee_recipient.to_string()),
        Some(deactivate_period),
    );

    let config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(config, CampaignConfig {
        creation_fee_token: Addr::unchecked(creation_fee_token),
        creation_fee_amount: creation_fee_amount.clone(),
        creation_fee_recipient: Addr::unchecked(creation_fee_recipient),
        code_id: code_id.clone(),
        distribution_denom_whitelist: config.distribution_denom_whitelist.clone(),
        withdraw_fee_rate: withdraw_fee_rate.clone(),
        withdraw_fee_recipient: Addr::unchecked(withdraw_fee_recipient),
        deactivate_period: deactivate_period.clone(),
    });
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        default_sender(),
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
