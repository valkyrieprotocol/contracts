use cosmwasm_std::{Addr, Api, Env, MessageInfo, Response, Uint128, Decimal};

use valkyrie::common::ContractResult;
use valkyrie::fund_manager::execute_msgs::InstantiateMsg;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::{default_sender, TERRASWAP_ROUTER};
use valkyrie::test_constants::fund_manager::{ADMINS, fund_manager_env, MANAGING_TOKEN, CAMPAIGN_DEPOSIT_FEE_BURN_RATIO_PERCENT, CAMPAIGN_DEPOSIT_FEE_RECIPIENT};

use crate::executions::instantiate;
use crate::states::{ContractConfig, ContractState};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    admins: Vec<String>,
    managing_token: String,
    terraswap_router: String,
    campaign_deposit_fee_burn_ratio: Decimal,
    campaign_deposit_fee_recipient: String,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        admins,
        managing_token,
        terraswap_router,
        campaign_deposit_fee_burn_ratio,
        campaign_deposit_fee_recipient,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = fund_manager_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        ADMINS.iter().map(|v| v.to_string()).collect(),
        MANAGING_TOKEN.to_string(),
        TERRASWAP_ROUTER.to_string(),
        Decimal::percent(CAMPAIGN_DEPOSIT_FEE_BURN_RATIO_PERCENT),
        CAMPAIGN_DEPOSIT_FEE_RECIPIENT.to_string(),
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    let config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(config, ContractConfig {
        admins: ADMINS.iter().map(|v| deps.api.addr_validate(v).unwrap()).collect(),
        managing_token: Addr::unchecked(MANAGING_TOKEN),
        terraswap_router: Addr::unchecked(TERRASWAP_ROUTER),
        campaign_deposit_fee_burn_ratio: Decimal::percent(CAMPAIGN_DEPOSIT_FEE_BURN_RATIO_PERCENT),
        campaign_deposit_fee_recipient: Addr::unchecked(CAMPAIGN_DEPOSIT_FEE_RECIPIENT),
    });

    let state = ContractState::load(&deps.storage).unwrap();
    assert_eq!(state, ContractState {
        remain_allowance_amount: Uint128::zero(),
        campaign_deposit_fee_amount: Uint128::zero(),
    });
}
