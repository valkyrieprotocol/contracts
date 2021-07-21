use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{Addr, Uint128};

use crate::mock_querier::mock_dependencies;

#[test]
fn query_cw20_balance() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_balances(&[(
        &"liquidity0000".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(123u128))],
    )]);

    assert_eq!(
        Uint128::from(123u128),
        crate::cw20::query_cw20_balance(
            &deps.as_ref().querier,
            deps.as_ref().api,
            &Addr::unchecked("liquidity0000"),
            &Addr::unchecked(MOCK_CONTRACT_ADDR),
        )
        .unwrap()
    );
}

#[test]
fn compress_address() {
    let address = "terra1h8ljdmae7lx05kjj79c9ekscwsyjd3yr8wyvdn";
    let compressed_address = super::utils::compress_addr(&address.to_string());
    let decompressed_address = super::utils::decompress_addr(&compressed_address);

    assert_eq!(compressed_address.len(), 32);
    assert_eq!(address, decompressed_address);
}
