use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response};
use valkyrie::common::ContractResult;
use crate::executions::update_factory_config;
use valkyrie::test_utils::{contract_env, default_sender, expect_unauthorized_err};
use crate::tests::{governance_sender, CAMPAIGN_CODE_ID, CREATION_FEE_AMOUNT};
use crate::states::FactoryConfig;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    campaign_code_id: Option<u64>,
    creation_fee_amount: Option<Uint128>,
) -> ContractResult<Response> {
    update_factory_config(deps.as_mut(), env, info, campaign_code_id, creation_fee_amount)
}

pub fn will_success(
    deps: &mut CustomDeps,
    campaign_code_id: Option<u64>,
    creation_fee_amount: Option<Uint128>,
) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = governance_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        campaign_code_id,
        creation_fee_amount,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let campaign_code_id = CAMPAIGN_CODE_ID + 1;
    let creation_fee_amount = CREATION_FEE_AMOUNT + Uint128(1);

    will_success(&mut deps, Some(campaign_code_id), Some(creation_fee_amount));

    let factory_config = FactoryConfig::load(&deps.storage).unwrap();
    assert_eq!(factory_config.campaign_code_id, campaign_code_id);
    assert_ne!(factory_config.campaign_code_id, CAMPAIGN_CODE_ID);
    assert_eq!(factory_config.creation_fee_amount, creation_fee_amount);
    assert_ne!(factory_config.creation_fee_amount, CREATION_FEE_AMOUNT);
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
    );

    expect_unauthorized_err(&result);
}