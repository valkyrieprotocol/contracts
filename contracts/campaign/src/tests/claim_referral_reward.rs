use cosmwasm_std::{Addr, CosmosMsg, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg, to_binary};
use cosmwasm_std::testing::mock_info;

use valkyrie::common::ContractResult;
use valkyrie::mock_querier::{custom_deps, CustomDeps};
use valkyrie::test_constants::campaign::{campaign_env, REFERRAL_REWARD_AMOUNTS};
use valkyrie::test_utils::expect_generic_err;

use crate::executions::claim_referral_reward;
use crate::states::{Actor, CampaignState};
use valkyrie::campaign::enumerations::Referrer;
use cw20::Cw20ExecuteMsg;
use valkyrie::test_constants::VALKYRIE_TOKEN;

pub fn exec(deps: &mut CustomDeps, env: Env, info: MessageInfo) -> ContractResult<Response> {
    claim_referral_reward(deps.as_mut(), env, info)
}

pub fn will_success(deps: &mut CustomDeps, sender: &str) -> (Env, MessageInfo, Response) {
    let env = campaign_env();
    let info = mock_info(sender, &[]);

    let response = exec(deps, env.clone(), info.clone()).unwrap();

    (env, info, response)
}

#[test]
fn succeed() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 1000, 1000);

    let referrer = Addr::unchecked("Referrer");
    super::participate::will_success(&mut deps, referrer.as_str(), None);
    super::participate::will_success(&mut deps, "Participator", Some(Referrer::Address(referrer.to_string())));

    let (_, info, response) = will_success(&mut deps, referrer.as_str());
    assert_eq!(response.messages, vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VALKYRIE_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: REFERRAL_REWARD_AMOUNTS[0],
            }).unwrap(),
        })),
    ]);

    let participation = Actor::load(&deps.storage, &referrer).unwrap();
    assert_eq!(participation.referral_reward_amount, Uint128::zero());
    assert_eq!(participation.cumulative_referral_reward_amount, REFERRAL_REWARD_AMOUNTS[0]);

    let campaign_state = CampaignState::load(&deps.storage).unwrap();
    assert_eq!(
        campaign_state.locked_balance(&cw20::Denom::Cw20(Addr::unchecked(VALKYRIE_TOKEN))),
        Uint128::zero(),
    );
}

#[test]
fn failed_no_reward() {
    let mut deps = custom_deps();

    super::instantiate::default(&mut deps);
    super::update_activation::will_success(&mut deps, true);
    super::add_reward_pool::will_success(&mut deps, 1000, 1000);

    let referrer = Addr::unchecked("Referrer");
    super::participate::will_success(&mut deps, referrer.as_str(), None);
    super::participate::will_success(&mut deps, "Participator", Some(Referrer::Address(referrer.to_string())));

    will_success(&mut deps, referrer.as_str());

    let result = exec(
        &mut deps,
        campaign_env(),
        mock_info(referrer.as_str(), &[]),
    );
    expect_generic_err(&result, "Not exist referral reward");
}
