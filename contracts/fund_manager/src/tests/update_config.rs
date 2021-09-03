use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, Decimal};
use valkyrie::common::ContractResult;
use crate::executions::update_config;
use valkyrie::test_utils::expect_unauthorized_err;
use crate::states::ContractConfig;
use valkyrie::test_constants::fund_manager::fund_manager_env;
use valkyrie::test_constants::governance::governance_sender;
use valkyrie::test_constants::default_sender;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admins: Option<Vec<String>>,
    terraswap_router: Option<String>,
    campaign_deposit_fee_burn_ratio: Option<Decimal>,
    campaign_deposit_fee_recipient: Option<String>,
) -> ContractResult<Response> {
    update_config(
        deps.as_mut(),
        env,
        info,
        admins,
        terraswap_router,
        campaign_deposit_fee_burn_ratio,
        campaign_deposit_fee_recipient,
    )
}

pub fn will_success(
    deps: &mut CustomDeps,
    admins: Option<Vec<String>>,
    terraswap_router: Option<String>,
    campaign_deposit_fee_burn_ratio: Option<Decimal>,
    campaign_deposit_fee_recipient: Option<String>,
) -> (Env, MessageInfo, Response) {
    let env = fund_manager_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        admins,
        terraswap_router,
        campaign_deposit_fee_burn_ratio,
        campaign_deposit_fee_recipient,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let admins = vec!["Admin1".to_string(), "Admins2".to_string()];
    let terraswap_router = "TerraSwapRouterChanged";
    let campaign_deposit_fee_burn_ratio = Decimal::percent(9);
    let campaign_deposit_fee_recipient = "ChangedRecipient";

    will_success(
        &mut deps,
        Some(admins.clone()),
        Some(terraswap_router.to_string()),
        Some(campaign_deposit_fee_burn_ratio),
        Some(campaign_deposit_fee_recipient.to_string()),
    );

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config.admins, admins.iter().map(|v| Addr::unchecked(v)).collect::<Vec<Addr>>());
    assert_eq!(config.terraswap_router, Addr::unchecked(terraswap_router));
    assert_eq!(config.campaign_deposit_fee_burn_ratio, campaign_deposit_fee_burn_ratio);
    assert_eq!(config.campaign_deposit_fee_recipient, Addr::unchecked(campaign_deposit_fee_recipient));
}

#[test]
fn failed_invalid_permission() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        fund_manager_env(),
        default_sender(),
        None,
        None,
        None,
        None,
    );
    expect_unauthorized_err(&result);
}
