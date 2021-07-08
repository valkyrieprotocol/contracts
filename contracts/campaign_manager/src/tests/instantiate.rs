use cosmwasm_std::{Decimal, Env, MessageInfo, Response, Uint128, Addr};
use cosmwasm_std::testing::mock_env;

use valkyrie::campaign_manager::execute_msgs::{BoosterConfigInitMsg, CampaignConfigInitMsg, ContractConfigInitMsg, InstantiateMsg};
use valkyrie::common::{ContractResult, Denom};
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_utils::default_sender;

use crate::executions::instantiate;
use crate::tests::{ACTIVITY_BOOSTER_MULTIPLIER_PERCENT, ACTIVITY_BOOSTER_RATIO_PERCENT, CAMPAIGN_CODE_ID, CAMPAIGN_DEACTIVATE_PERIOD, CREATION_FEE_AMOUNT, DISTRIBUTION_DENOM_WHITELIST_NATIVE, DISTRIBUTION_DENOM_WHITELIST_TOKEN, DROP_BOOSTER_RATIO_PERCENT, FUND_MANAGER, GOVERNANCE, MIN_PARTICIPATION_COUNT, PLUS_BOOSTER_RATIO_PERCENT, TOKEN_CONTRACT, WITHDRAW_FEE_RATE_PERCENT};
use crate::states::{ContractConfig, CampaignConfig, BoosterConfig};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: String,
    fund_manager: String,
    creation_fee_token: String,
    creation_fee_amount: Uint128,
    creation_fee_recipient: String,
    code_id: u64,
    distribution_denom_whitelist: Vec<Denom>,
    withdraw_fee_rate: Decimal,
    withdraw_fee_recipient: String,
    deactivate_period: u64,
    booster_token: String,
    drop_booster_ratio: Decimal,
    activity_booster_ratio: Decimal,
    plus_booster_ratio: Decimal,
    activity_booster_multiplier: Decimal,
    min_participation_count: u64,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        contract_config: ContractConfigInitMsg {
            governance,
            fund_manager,
        },
        campaign_config: CampaignConfigInitMsg {
            creation_fee_token,
            creation_fee_amount,
            creation_fee_recipient,
            code_id,
            distribution_denom_whitelist,
            withdraw_fee_rate,
            withdraw_fee_recipient,
            deactivate_period,
        },
        booster_config: BoosterConfigInitMsg {
            booster_token,
            drop_booster_ratio,
            activity_booster_ratio,
            plus_booster_ratio,
            activity_booster_multiplier,
            min_participation_count,
        },
    };

    instantiate(
        deps.as_mut(),
        env,
        info,
        msg,
    )
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = default_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        GOVERNANCE.to_string(),
        FUND_MANAGER.to_string(),
        TOKEN_CONTRACT.to_string(),
        CREATION_FEE_AMOUNT,
        FUND_MANAGER.to_string(),
        CAMPAIGN_CODE_ID,
        vec![
            Denom::Native(DISTRIBUTION_DENOM_WHITELIST_NATIVE.to_string()),
            Denom::Token(DISTRIBUTION_DENOM_WHITELIST_TOKEN.to_string()),
        ],
        Decimal::percent(WITHDRAW_FEE_RATE_PERCENT),
        FUND_MANAGER.to_string(),
        CAMPAIGN_DEACTIVATE_PERIOD,
        TOKEN_CONTRACT.to_string(),
        Decimal::percent(DROP_BOOSTER_RATIO_PERCENT),
        Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT),
        Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT),
        Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
        MIN_PARTICIPATION_COUNT,
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    default(&mut deps);

    let contract_config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(contract_config, ContractConfig {
        governance: Addr::unchecked(GOVERNANCE),
        fund_manager: Addr::unchecked(FUND_MANAGER),
    });

    let campaign_config = CampaignConfig::load(&deps.storage).unwrap();
    assert_eq!(campaign_config, CampaignConfig {
        creation_fee_token: Addr::unchecked(TOKEN_CONTRACT),
        creation_fee_amount: CREATION_FEE_AMOUNT,
        creation_fee_recipient: Addr::unchecked(FUND_MANAGER),
        code_id: CAMPAIGN_CODE_ID,
        distribution_denom_whitelist: vec![
            cw20::Denom::Native(DISTRIBUTION_DENOM_WHITELIST_NATIVE.to_string()),
            cw20::Denom::Cw20(Addr::unchecked(DISTRIBUTION_DENOM_WHITELIST_TOKEN)),
        ],
        withdraw_fee_rate: Decimal::percent(WITHDRAW_FEE_RATE_PERCENT),
        withdraw_fee_recipient: Addr::unchecked(FUND_MANAGER),
        deactivate_period: CAMPAIGN_DEACTIVATE_PERIOD,
    });

    let booster_config = BoosterConfig::load(&deps.storage).unwrap();
    assert_eq!(booster_config, BoosterConfig {
        booster_token: Addr::unchecked(TOKEN_CONTRACT),
        drop_ratio: Decimal::percent(DROP_BOOSTER_RATIO_PERCENT),
        activity_ratio: Decimal::percent(ACTIVITY_BOOSTER_RATIO_PERCENT),
        plus_ratio: Decimal::percent(PLUS_BOOSTER_RATIO_PERCENT),
        activity_multiplier: Decimal::percent(ACTIVITY_BOOSTER_MULTIPLIER_PERCENT),
        min_participation_count: MIN_PARTICIPATION_COUNT,
    });
}