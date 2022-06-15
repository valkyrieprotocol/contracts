use cosmwasm_std::{Addr, StdError, Uint128};
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

#[test]
fn compress_contract_address() {
    let address = "terra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqxl5qul";
    let compressed_address = super::utils::compress_addr(&address.to_string()).unwrap();
    let decompressed_address = super::utils::decompress_addr(&compressed_address).unwrap();

    assert_eq!(compressed_address.len(), 76);
    assert_eq!(address, decompressed_address);
}

#[test]
fn invalid_compressed_address() {
    let compressed_address = "awgeilwuhfelwiefj";
    let decompressed_address = super::utils::decompress_addr(&compressed_address).unwrap_err();

    assert_eq!(decompressed_address, StdError::generic_err("Invalid compressed addr."));
}