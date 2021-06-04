use cosmwasm_std::{Addr, Api, Uint128};
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;

use crate::mock_querier::mock_dependencies;

#[test]
fn _test_query_cw20_balance() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_balances(&[(
        &"liquidity0000".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(123u128))],
    )]);

    assert_eq!(
        Uint128(123u128),
        crate::cw20::query_cw20_balance(
            &deps.as_ref().querier,
            deps.as_ref().api,
            &deps.api.addr_canonicalize(Addr::unchecked("liquidity0000").as_str()).unwrap(),
            &Addr::unchecked(MOCK_CONTRACT_ADDR),
        )
            .unwrap()
    );
}