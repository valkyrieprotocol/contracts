use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use crate::valkyrie::executions;
use crate::valkyrie::states::{ValkyrieConfig, CampaignCode};
use cosmwasm_std::Addr;
use crate::tests::init;

const CODE_ID: u64 = 1u64;
const SOURCE_CODE_URL: String = String::from("https://source-code-url.com/path.git");
const DESCRIPTION: String = String::from("description...");
const MAINTAINER: String = String::from("tester@terra.money");

#[test]
fn instantiate_succeed() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("", &[]);

    // Execute
    executions::instantiate(deps.as_mut(), env.clone(), info.clone()).unwrap();

    // Validate
    let valkyrie_config = ValkyrieConfig::load(&deps.storage).unwrap();
    assert!(valkyrie_config.campaign_code_whitelist.is_empty());
    assert_eq!(valkyrie_config.boost_contract, None);
}

#[test]
fn update_config_succeed() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info(env.contract.address.as_str(), &[]);

    init(deps.as_mut(), env.clone(), info.clone());

    let boost_contract = Some(Addr::unchecked("boost_contract"));

    // Execute
    executions::update_config(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        boost_contract.clone(),
    ).unwrap();

    // Validate
    let valkyrie_config = ValkyrieConfig::load(&deps.storage).unwrap();
    assert_eq!(valkyrie_config.boost_contract, boost_contract);
}

#[test]
fn update_config_permission() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("NotAdminAddress", &[]);

    init(deps.as_mut(), env.clone(), info.clone());

    let boost_contract = Some(Addr::unchecked("boost_contract"));

    // Execute
    let result = executions::update_config(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        boost_contract.clone()
    );

    // Validate
    assert!(result.is_err());
}

#[test]
fn add_campaign_code_whitelist_succeed() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info(env.contract.address.as_str(), &[]);

    init(deps.as_mut(), env.clone(), info.clone());

    // Execute
    executions::add_campaign_code_whitelist(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CODE_ID.clone(),
        SOURCE_CODE_URL.clone(),
        DESCRIPTION.clone(),
        Some(MAINTAINER.clone()),
    ).unwrap();

    // Validate
    let campaign_code = CampaignCode::load(&deps.storage, &code_id).unwrap();

    assert_eq!(campaign_code.code_id, CODE_ID);
    assert_eq!(campaign_code.source_code_url, SOURCE_CODE_URL);
    assert_eq!(campaign_code.description, DESCRIPTION);
    assert_eq!(campaign_code.maintainer, Some(MAINTAINER));

    let valkyrie_config = ValkyrieConfig::load(&deps.storage).unwrap();
    assert!(valkyrie_config.campaign_code_whitelist.contains(&code_id));
}

#[test]
fn add_campaign_code_whitelist_permission() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("NotAdminAddress", &[]);

    init(deps.as_mut(), env.clone(), info.clone());

    // Execute
    let result = executions::add_campaign_code_whitelist(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CODE_ID.clone(),
        SOURCE_CODE_URL.clone(),
        DESCRIPTION.clone(),
        Some(MAINTAINER.clone()),
    );

    // Validate
    assert!(result.is_err());
}

#[test]
fn remove_campaign_code_whitelist() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info(env.contract.address.as_str(), &[]);

    init(deps.as_mut(), env.clone(), info.clone());

    executions::add_campaign_code_whitelist(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CODE_ID.clone(),
        SOURCE_CODE_URL.clone(),
        DESCRIPTION.clone(),
        Some(MAINTAINER.clone()),
    ).unwrap();

    // Execute
    executions::remove_campaign_code_whitelist(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CODE_ID.clone(),
    ).unwrap();

    // Validate
    assert!(CampaignCode::load(&deps.storage, &CODE_ID).is_ok());

    let valkyrie_config = ValkyrieConfig::load(&deps.storage).unwrap();
    assert!(!valkyrie_config.campaign_code_whitelist.contains(&CODE_ID));
}

#[test]
fn remove_campaign_code_permission() {
    // Initialize
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("NotAdminAddress", &[]);

    init(deps.as_mut(), env.clone(), info.clone());

    // Execute
    let result = executions::remove_campaign_code_whitelist(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CODE_ID.clone(),
    );

    // Validate
    assert!(result.is_err());
}