use cosmwasm_std::{Addr, Uint128};
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use crate::mock_querier::custom_deps;

#[test]
fn query_cw20_balance() {
    let mut deps = custom_deps();

    deps.querier.with_token_balances(&[(
        &"liquidity0000".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(123u128))],
    )]);

    assert_eq!(
        Uint128::from(123u128),
        crate::cw20::query_cw20_balance(
            &deps.as_ref().querier,
            &Addr::unchecked("liquidity0000"),
            &Addr::unchecked(MOCK_CONTRACT_ADDR),
        )
            .unwrap()
    );
}

#[test]
fn compress_address() {
    let address = "terra1h8ljdmae7lx05kjj79c9ekscwsyjd3yr8wyvdn";
    let compressed_address = super::utils::compress_addr(&address.to_string()).unwrap();
    let decompressed_address = super::utils::decompress_addr(&compressed_address).unwrap();

    assert_eq!(compressed_address.len(), 32);
    assert_eq!(address, decompressed_address);
}