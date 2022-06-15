use cosmwasm_std::{Addr, Env, Event, Reply, Response, SubMsgResponse, SubMsgResult};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign_manager::campaign_manager_env;

use crate::executions::{created_campaign, REPLY_CREATE_CAMPAIGN};
use crate::states::{Campaign, CreateCampaignContext};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    result: SubMsgResult,
) -> ContractResult<Response> {
    created_campaign(deps.as_mut(), env, Reply {
        id: REPLY_CREATE_CAMPAIGN,
        result,
    })
}

#[test]
fn succeed_success_reply() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::create_campaign::default(&mut deps);

    let context = CreateCampaignContext::load(&deps.storage).unwrap();

    let campaign_address = Addr::unchecked("terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n");

    let env = campaign_manager_env();
    let result = exec(
        &mut deps,
        env.clone(),
        SubMsgResult::Ok(SubMsgResponse {
            events: vec![
                Event::new("instantiate_contract")
                    .add_attribute("contract_address", campaign_address.to_string()),
            ],
            data: None,
        }),
    );

    assert!(result.is_ok());
    assert!(CreateCampaignContext::may_load(&deps.storage).unwrap().is_none());

    let campaign = Campaign::load(&deps.storage, &campaign_address).unwrap();
    assert_eq!(campaign, Campaign {
        code_id: context.code_id,
        address: campaign_address,
        creator: context.creator,
        created_height: env.block.height,
    });
}