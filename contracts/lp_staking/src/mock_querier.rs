use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Api, Coin, ContractResult, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::BalanceResponse;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_balance: Uint128,
    tax: (Decimal, Uint128),
}

pub fn mock_dependencies_with_querier(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = Addr::unchecked(MOCK_CONTRACT_ADDR);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: WasmMockQuerier::new(
            MockQuerier::new(&[(&contract_addr.as_str(), contract_balance)]),
            MockApi::default(),
        ),
    }
}

impl WasmMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<TerraQueryWrapper>, _api: A) -> Self {
        WasmMockQuerier {
            base,
            token_balance: Uint128::zero(),
            tax: (Decimal::percent(1), Uint128::new(1000000)),
        }
    }
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if route == &TerraRoute::Treasury {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse { rate: self.tax.0 };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { .. } => {
                            let res = TaxCapResponse { cap: self.tax.1 };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: _,
            }) => {
                if contract_addr == "lp_token" {
                    let res = BalanceResponse {
                        balance: Uint128::from(0u64),
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&res)))
                } else {
                    self.base.handle_query(request)
                }
            }
            QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: _,
                key,
            }) => {
                let key: &[u8] = key.as_slice();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();
                if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&self.token_balance).unwrap(),
                    )))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}
