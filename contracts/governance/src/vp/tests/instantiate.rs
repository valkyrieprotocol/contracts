use cosmwasm_std::{Env, MessageInfo, Response};

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::default_sender;
use valkyrie::test_constants::governance::*;

use valkyrie::governance::execute_msgs::{TicketConfigInitMsg};
use crate::vp::executions::instantiate;
use crate::vp::states::TicketConfig;

pub fn exec(
    deps: &mut CustomDeps,
    msg: TicketConfigInitMsg,
) -> ContractResult<Response> {
    instantiate(deps.as_mut(), msg)
}

pub fn default(deps: &mut CustomDeps) -> (Env, MessageInfo, Response) {
    let env = governance_env();
    let info = default_sender();

    let response = exec(
        deps,
        TicketConfigInitMsg {
            ticket_token: "abcd".to_string(),
            distribution_schedule: vec![TICKET_DIST_SCHEDULE]
        },
    ).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    default(&mut deps);

    // Validate
    let ticket_config = TicketConfig::load(&deps.storage).unwrap();
    assert_eq!(ticket_config.ticket_token, "abcd".to_string());
    assert_eq!(ticket_config.distribution_schedule, vec![TICKET_DIST_SCHEDULE]);
}
