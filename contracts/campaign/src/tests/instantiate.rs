use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Uint128, Response, Addr};
use valkyrie::campaign::enumerations::Denom;
use valkyrie::common::ContractResult;
use crate::executions::instantiate;
use valkyrie::campaign::execute_msgs::InstantiateMsg;
use cosmwasm_std::testing::mock_env;
use crate::tests::{factory_sender, GOVERNANCE, DISTRIBUTOR, TOKEN_CONTRACT, FACTORY, BURN_CONTRACT, CAMPAIGN_TITLE, CAMPAIGN_DESCRIPTION, CAMPAIGN_URL, CAMPAIGN_PARAMETER_KEY, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE, CAMPAIGN_DISTRIBUTION_AMOUNTS, CAMPAIGN_ADMIN};
use crate::states::{ContractConfig, CampaignInfo, CampaignState, DistributionConfig};
use cw20::Denom as Cw20Denom;

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    info: MessageInfo,
    governance: String,
    distributor: String,
    token_contract: String,
    factory: String,
    burn_contract: String,
    title: String,
    description: String,
    url: String,
    parameter_key: String,
    distribution_denom: Denom,
    distribution_amounts: Vec<Uint128>,
    admin: String,
    creator: String,
) -> ContractResult<Response> {
    let msg = InstantiateMsg {
        governance,
        distributor,
        token_contract,
        factory,
        burn_contract,
        title,
        url,
        description,
        parameter_key,
        distribution_denom,
        distribution_amounts,
        admin,
        creator,
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
    admin: String,
    creator: String,
) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = factory_sender();

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        GOVERNANCE.to_string(),
        DISTRIBUTOR.to_string(),
        TOKEN_CONTRACT.to_string(),
        FACTORY.to_string(),
        BURN_CONTRACT.to_string(),
        title,
        description,
        url,
        parameter_key,
        distribution_denom,
        distribution_amounts,
        admin,
        creator,
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
        CAMPAIGN_ADMIN.to_string(),
        CAMPAIGN_ADMIN.to_string(),
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
        distributor: Addr::unchecked(DISTRIBUTOR),
        token_contract: Addr::unchecked(TOKEN_CONTRACT),
        factory: Addr::unchecked(FACTORY),
        burn_contract: Addr::unchecked(BURN_CONTRACT),
    });

    let campaign_info = CampaignInfo::load(&deps.storage).unwrap();
    assert_eq!(campaign_info, CampaignInfo {
        title: CAMPAIGN_TITLE.to_string(),
        description: CAMPAIGN_DESCRIPTION.to_string(),
        url: CAMPAIGN_URL.to_string(),
        parameter_key: CAMPAIGN_PARAMETER_KEY.to_string(),
        creator: Addr::unchecked(CAMPAIGN_ADMIN),
        created_at: env.block.time,
        created_block: env.block.height,
    });

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(campaign_state, CampaignState {
        participation_count: 0,
        cumulative_distribution_amount: vec![],
        locked_balance: vec![],
        active_flag: false,
        last_active_block: None,
    });

    let distribution_config = DistributionConfig::load(&deps.storage).unwrap();
    assert_eq!(distribution_config, DistributionConfig {
        denom: Cw20Denom::Native(CAMPAIGN_DISTRIBUTION_DENOM_NATIVE.to_string()),
        amounts: CAMPAIGN_DISTRIBUTION_AMOUNTS.to_vec(),
    });
}
