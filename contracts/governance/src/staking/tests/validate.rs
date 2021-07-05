use cosmwasm_std::{attr, Uint128};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_env};

use valkyrie::mock_querier::custom_deps;

use crate::staking::queries::get_staker_state;
use crate::staking::tests::stake_governance_token::STAKER1;
use crate::tests::{init_default, GOVERNANCE_TOKEN};

#[test]
fn share_calculation() {
    let mut deps = custom_deps(&[]);

    init_default(deps.as_mut());

    super::stake_governance_token::will_success(&mut deps, STAKER1, Uint128(100));

    deps.querier.plus_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(MOCK_CONTRACT_ADDR, &Uint128(100))],
    )]);

    let (_, _, response) = super::stake_governance_token::will_success(
        &mut deps,
        STAKER1,
        Uint128(100),
    );

    assert_eq!(response.attributes, vec![
        attr("action", "stake_governance_token"),
        attr("sender", STAKER1),
        attr("share", "50"),
        attr("amount", "100"),
    ]);

    let (_, _, response) = super::unstake_governance_token::will_success(
        &mut deps,
        STAKER1,
        Some(Uint128(100)),
    );

    assert_eq!(response.attributes, vec![
        attr("action", "unstake_governance_token"),
        attr("unstake_amount", "100"),
        attr("unstake_share", "50")
    ]);

    let staker_state = get_staker_state(deps.as_ref(), mock_env(), STAKER1.to_string()).unwrap();
    assert_eq!(staker_state.share, Uint128(100));
    assert_eq!(staker_state.balance, Uint128(200));
}