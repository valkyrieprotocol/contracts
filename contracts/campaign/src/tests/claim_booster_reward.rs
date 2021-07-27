use valkyrie::mock_querier::{CustomDeps, custom_deps};
use cosmwasm_std::{Env, MessageInfo, Response, Addr, coin, CosmosMsg, WasmMsg, to_binary, Uint128, Decimal, SubMsg};
use valkyrie::common::ContractResult;
use crate::executions::claim_booster_reward;
use valkyrie::test_utils::{contract_env, expect_generic_err, expect_not_found_err};
use cosmwasm_std::testing::mock_info;
use crate::tests::{CAMPAIGN_DISTRIBUTION_DENOM_NATIVE, FUND_MANAGER};
use valkyrie::campaign::enumerations::Referrer;
use valkyrie::fund_manager::execute_msgs::ExecuteMsg;
use crate::states::Participation;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    claim_booster_reward(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, sender: &str) -> (Env, MessageInfo, Response) {
    let env = contract_env();
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed_booster() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    deps.querier.with_voting_powers(&[
        (&"Participator4".to_string(), &Decimal::percent(10)),
    ]);
    super::participate::will_success(&mut deps, "Participator1", None);
    super::participate::will_success(&mut deps, "Participator4", Some(Referrer::Address("Participator1".to_string())));
    super::enable_booster::default(&mut deps);
    super::participate::will_success(&mut deps, "Participator2", Some(Referrer::Address("Participator1".to_string())));
    deps.querier.with_voting_powers(&[
        (&"Participator3".to_string(), &Decimal::percent(10)),
    ]);
    super::participate::will_success(&mut deps, "Participator3", Some(Referrer::Address("Participator2".to_string())));

    //drop[0] + drop[1] + activity[1] + activity[2]
    let (_, _, response) = will_success(&mut deps, "Participator1");
    expect_transfer(&response, "Participator1", Uint128::from(250u64) + Uint128::from(150u64) + Uint128::from(120u64) + Uint128::from(80u64));

    //drop[0]
    let (_, _, response) = will_success(&mut deps, "Participator4");
    expect_transfer(&response, "Participator4", Uint128::from(250u64));

    super::disable_booster::will_success(&mut deps);

    //activity[0] + activity[1]
    let (_, _, response) = will_success(&mut deps, "Participator2");
    expect_transfer(&response, "Participator2", Uint128::from(200u64) + Uint128::from(120u64));

    //activity[0] + plus
    let (_, _, response) = will_success(&mut deps, "Participator3");
    expect_transfer(&response, "Participator3", Uint128::from(200u64) + Uint128::from(100u64));
}

#[test]
fn succeed_multiple_booster() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    deps.querier.with_voting_powers(&[
        (&"Participator4".to_string(), &Decimal::percent(10)),
    ]);
    super::participate::will_success(&mut deps, "Participator1", None);
    super::participate::will_success(&mut deps, "Participator4", Some(Referrer::Address("Participator1".to_string())));
    super::enable_booster::default(&mut deps);
    super::participate::will_success(&mut deps, "Participator2", Some(Referrer::Address("Participator1".to_string())));
    deps.querier.with_voting_powers(&[
        (&"Participator3".to_string(), &Decimal::percent(10)),
    ]);
    super::participate::will_success(&mut deps, "Participator3", Some(Referrer::Address("Participator2".to_string())));

    //drop[0] + drop[1] + activity[1] + activity[2]
    let (_, _, response) = will_success(&mut deps, "Participator1");
    expect_transfer(&response, "Participator1", Uint128::from(250u64) + Uint128::from(150u64) + Uint128::from(120u64) + Uint128::from(80u64));

    //drop[0]
    let (_, _, response) = will_success(&mut deps, "Participator4");
    expect_transfer(&response, "Participator4", Uint128::from(250u64));

    //activity[0] + plus
    let (_, _, response) = will_success(&mut deps, "Participator3");
    expect_transfer(&response, "Participator3", Uint128::from(200u64) + Uint128::from(100u64));

    super::disable_booster::will_success(&mut deps);
    super::participate::will_success(&mut deps, "Participator5", Some(Referrer::Address("Participator3".to_string())));
    super::enable_booster::default(&mut deps);
    super::participate::will_success(&mut deps, "Participator6", Some(Referrer::Address("Participator5".to_string())));

    //activity[0] + activity[1] + drop_2[0] + drop_2[1] + drop_2[2]
    let (_, _, response) = will_success(&mut deps, "Participator2");
    expect_transfer(&response, "Participator2", Uint128::from(200u64) + Uint128::from(120u64) + Uint128::from(200u64));

    //drop_2[0] + drop_2[1] + activity_2[2]
    let (_, _, response) = will_success(&mut deps, "Participator3");
    expect_transfer(&response, "Participator3", Uint128::from(100u64) + Uint128::from(60u64) + Uint128::from(32u64));

    //drop_2[0] + activity_2[1]
    let (_, _, response) = will_success(&mut deps, "Participator5");
    expect_transfer(&response, "Participator5", Uint128::from(100u64) + Uint128::from(48u64));

    super::disable_booster::will_success(&mut deps);

    //activity_2[0]
    let (_, _, response) = will_success(&mut deps, "Participator6");
    expect_transfer(&response, "Participator6", Uint128::from(80u64));
}

#[test]
fn test_claimable_state_after_claim() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);

    deps.querier.with_voting_powers(&[
        (&"Participator4".to_string(), &Decimal::percent(10)),
    ]);

    super::participate::will_success(&mut deps, "Participator1", None);
    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator1")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(1, true)]);

    super::participate::will_success(&mut deps, "Participator2", Some(Referrer::Address("Participator1".to_string())));
    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator2")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(1, true)]);

    super::enable_booster::default(&mut deps);
    will_success(&mut deps, "Participator1");

    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator1")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(1, false)]);

    super::participate::will_success(&mut deps, "Participator3", None);
    will_success(&mut deps, "Participator3");
    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator3")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(2, true)]);

    super::disable_booster::will_success(&mut deps);

    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator2")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(1, true)]);

    will_success(&mut deps, "Participator2");

    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator2")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(1, false)]);

    super::participate::will_success(&mut deps, "Participator4", None);
    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator4")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(2, true)]);

    let result = exec(&mut deps, contract_env(), mock_info("Participator3", &[]));
    expect_generic_err(&result, "Not exist booster reward");

    super::enable_booster::default(&mut deps);

    will_success(&mut deps, "Participator3");
    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator3")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(2, false)]);

    will_success(&mut deps, "Participator4");
    let participation = Participation::load(&deps.storage, &Addr::unchecked("Participator4")).unwrap();
    assert_eq!(participation.drop_booster_claimable, vec![(2, false)]);
}

#[test]
fn failed_no_reward() {
    let mut deps = custom_deps(&[
        coin(1000, CAMPAIGN_DISTRIBUTION_DENOM_NATIVE),
    ]);

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::participate::will_success(&mut deps, "Part", None);
    super::enable_booster::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info("Participator", &[]),
    );
    expect_not_found_err(&result);

    super::participate::will_success(&mut deps, "Participator", None);

    will_success(&mut deps, "Participator");

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info("Participator", &[]),
    );
    expect_generic_err(&result, "Not exist booster reward");
}

#[test]
fn failed_not_boosted() {
    let mut deps = custom_deps(&[]);

    super::instantiate::default(&mut deps);

    let result = exec(
        &mut deps,
        contract_env(),
        mock_info("Participator", &[]),
    );
    expect_generic_err(&result, "Not boosted campaign");
}

fn expect_transfer(response: &Response, recipient: &str, amount: Uint128) {
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: FUND_MANAGER.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount,
            }).unwrap(),
        })),
    ]);
}
