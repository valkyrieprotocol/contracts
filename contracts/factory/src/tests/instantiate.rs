use cosmwasm_std::{Api, Decimal, Env, MessageInfo, Response, Uint128};
use cosmwasm_std::testing::mock_env;

use valkyrie::common::ContractResult;
use valkyrie::factory::execute_msgs::InstantiateMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::default_sender;

use crate::executions::instantiate;
use crate::states::{CampaignConfig, FactoryConfig};
use crate::tests::*;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: String,
    token_contract: String,
    distributor: String,
    burn_contract: String,
    campaign_code_id: u64,
    creation_fee_amount: Uint128,
    reward_withdraw_burn_rate: Decimal,
    campaign_deactivate_period: u64,
) -> ContractResult<Response> {
    let init_msg = InstantiateMsg {
        governance,
        token_contract,
        distributor,
        burn_contract,
        campaign_code_id,
        creation_fee_amount,
        reward_withdraw_burn_rate,
        campaign_deactivate_period,
    };

    instantiate(deps.as_mut(), env, info, init_msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        GOVERNANCE.to_string(),
        TOKEN_CONTRACT.to_string(),
        DISTRIBUTOR.to_string(),
        BURN_CONTRACT.to_string(),
        CAMPAIGN_CODE_ID,
        CREATION_FEE_AMOUNT,
        Decimal::percent(REWARD_WITHDRAW_BURN_RATE_PERCENT),
        CAMPAIGN_DEACTIVATE_PERIOD,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    default(&mut deps);

    let factory_config = FactoryConfig::load(&deps.storage).unwrap();
    assert_eq!(factory_config, FactoryConfig {
        governance: deps.api.addr_validate(GOVERNANCE).unwrap(),
        token_contract: deps.api.addr_validate(TOKEN_CONTRACT).unwrap(),
        distributor: deps.api.addr_validate(DISTRIBUTOR).unwrap(),
        burn_contract: deps.api.addr_validate(BURN_CONTRACT).unwrap(),
        campaign_code_id: CAMPAIGN_CODE_ID,
        creation_fee_amount: CREATION_FEE_AMOUNT,
    });

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config, CampaignConfig {
        reward_withdraw_burn_rate: Decimal::percent(REWARD_WITHDRAW_BURN_RATE_PERCENT),
        campaign_deactivate_period: CAMPAIGN_DEACTIVATE_PERIOD,
    });
}