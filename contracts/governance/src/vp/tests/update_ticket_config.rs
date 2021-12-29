use cosmwasm_std::{Addr, Uint128};
use valkyrie::mock_querier::{custom_deps};
use valkyrie::test_constants::governance::{TICKET_DIST_SCHEDULE, TICKET_TOKEN};
use crate::vp::executions::update_ticket_config;
use crate::vp::states::TicketConfig;

use crate::tests::init_default;

#[test]
fn update_ticket_config_test() {
    let mut deps = custom_deps();

    let (env, mut info) = init_default(deps.as_mut());

    info.sender = env.clone().contract.address;

    let _result = update_ticket_config(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        None,
        None,
    );

    let config = TicketConfig::load(&deps.storage).unwrap();
    assert_eq!(config.ticket_token, TICKET_TOKEN.to_string());
    assert_eq!(config.distribution_schedule, vec![TICKET_DIST_SCHEDULE]);

    let _result = update_ticket_config(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(Addr::unchecked("DDDD").to_string()),
        Some(vec![TICKET_DIST_SCHEDULE, (100, 200, Uint128::new(1234u128))])
    );

    let config = TicketConfig::load(&deps.storage).unwrap();
    assert_eq!(config.ticket_token, "DDDD".to_string());
    assert_eq!(config.distribution_schedule, vec![TICKET_DIST_SCHEDULE, (100, 200, Uint128::new(1234u128))]);

}