use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, ContractResult as CwContractResult, SubcallResponse, Response, Reply, Event, attr, Addr};
use valkyrie::common::ContractResult;
use crate::executions::{created_campaign, REPLY_CREATE_CAMPAIGN};
use valkyrie::test_utils::contract_env;
use crate::states::{CreateCampaignContext, Campaign};

pub fn exec(
    deps: &mut CustomDeps,
    env: Env,
    result: CwContractResult<SubcallResponse>,
) -> ContractResult<Response> {
    created_campaign(deps.as_mut(), env, Reply {
        id: REPLY_CREATE_CAMPAIGN,
        result,
    })
}

#[test]
fn succeed_success_reply() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);
    super::create_campaign::default(&mut deps);

    let context = CreateCampaignContext::load(&deps.storage).unwrap();

    let campaign_address = Addr::unchecked("CampaignContractAddress");

    let env = contract_env();
    let result = exec(
        &mut deps,
        env.clone(),
        CwContractResult::Ok(SubcallResponse {
            events: vec![
                Event {
                    kind: "instantiate_contract".to_string(),
                    attributes: vec![
                        attr("contract_address", campaign_address.to_string()),
                    ],
                },
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
        created_block: env.block.height,
    });
}