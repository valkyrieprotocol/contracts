use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, Addr, to_binary};
use valkyrie::common::{ContractResult, Denom, ExecutionMsg};
use crate::executions::instantiate;
use valkyrie::campaign::execute_msgs::CampaignConfigMsg;
use cosmwasm_std::testing::mock_env;
use crate::tests::{GOVERNANCE, CAMPAIGN_TITLE, CAMPAIGN_DESCRIPTION, CAMPAIGN_URL, CAMPAIGN_PARAMETER_KEY, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE, CAMPAIGN_DISTRIBUTION_AMOUNTS, CAMPAIGN_ADMIN, CAMPAIGN_MANAGER, FUND_MANAGER, campaign_manager_sender};
use crate::states::{ContractConfig, CampaignInfo, CampaignState, DistributionConfig, BoosterState};
use cw20::Denom as Cw20Denom;
use valkyrie::campaign_manager::execute_msgs::CampaignInstantiateMsg;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    url: String,
    parameter_key: String,
    distribution_denom: Denom,
    distribution_amounts: Vec<Uint128>,
    proxies: Vec<String>,
    executions: Vec<ExecutionMsg>,
) -> ContractResult<Response> {
    let config_msg = CampaignConfigMsg {
        title,
        url,
        description,
        parameter_key,
        distribution_denom,
        distribution_amounts,
    };

    let msg = CampaignInstantiateMsg {
        governance: GOVERNANCE.to_string(),
        campaign_manager: CAMPAIGN_MANAGER.to_string(),
        fund_manager: FUND_MANAGER.to_string(),
        admin: CAMPAIGN_ADMIN.to_string(),
        creator: CAMPAIGN_ADMIN.to_string(),
        proxies,
        executions,
        config_msg: to_binary(&config_msg)?,
    };

    instantiate(deps.as_mut(), env, info, msg)
}

pub fn will_success(
    deps: &mut CustomDeps,
    title: String,
    description: String,
    url: String,
    parameter_key: String,
    distribution_denom: Denom,
    distribution_amounts: Vec<Uint128>,
    proxies: Vec<String>,
    executions: Vec<ExecutionMsg>,
) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = campaign_manager_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        title,
        description,
        url,
        parameter_key,
        distribution_denom,
        distribution_amounts,
        proxies,
        executions,
    ).unwrap();

    (env, info, response)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    will_success(
        deps,
        CAMPAIGN_TITLE.to_string(),
        CAMPAIGN_DESCRIPTION.to_string(),
        CAMPAIGN_URL.to_string(),
        CAMPAIGN_PARAMETER_KEY.to_string(),
        Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        CAMPAIGN_DISTRIBUTION_AMOUNTS.to_vec(),
        vec![],
        vec![],
    )
}

#[test]
fn succeed() {
    let mut deps = custom_deps(&[]);

    let (env, _, _) = default(&mut deps);

    let contract_config = ContractConfig::load(&deps.storage).unwrap();
    assert_eq!(contract_config, ContractConfig {
        admin: Addr::unchecked(CAMPAIGN_ADMIN),
        governance: Addr::unchecked(GOVERNANCE),
        campaign_manager: Addr::unchecked(CAMPAIGN_MANAGER),
        fund_manager: Addr::unchecked(FUND_MANAGER),
        proxies: vec![],
    });

    let campaign_info = CampaignInfo::load(&deps.storage).unwrap();
    assert_eq!(campaign_info, CampaignInfo {
        title: CAMPAIGN_TITLE.to_string(),
        description: CAMPAIGN_DESCRIPTION.to_string(),
        url: CAMPAIGN_URL.to_string(),
        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
        executions: vec![],
        creator: Addr::unchecked(CAMPAIGN_ADMIN),
        created_at: env.block.time,
        created_height: env.block.height,
    });

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state, CampaignState {
        participation_count: 0,
        distance_counts: vec![],
        cumulative_distribution_amount: Uint128::zero(),
        locked_balance: Uint128::zero(),
        active_flag: false,
        last_active_height: None,
    });

    let distribution_config = DistributionConfig::load(&deps.storage).unwrap();
    assert_eq!(distribution_config, DistributionConfig {
        denom: Cw20Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        amounts: CAMPAIGN_DISTRIBUTION_AMOUNTS.to_vec(),
    });

    let booster_state = BoosterState::load(&deps.storage).unwrap();
    assert_eq!(booster_state, BoosterState {
        recent_booster_id: 0u64,
    });
}
