use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::collections::HashMap;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};
use valkyrie::governance::query_msgs::{
    QueryMsg as GovQueryMsg, VotingPowerResponse,
};
use valkyrie::factory::query_msgs::{QueryMsg as FactoryQueryMsg, CampaignConfigResponse};

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    voting_powers_querier: VotingPowerQuerier,
    tax_querier: TaxQuerier,
    valkyrie_config_querier: ValkyrieConfigQuerier,
}

#[derive(Clone, Default)]
pub struct VotingPowerQuerier {
    powers: HashMap<String, Decimal>,
}

impl VotingPowerQuerier {
    pub fn new(powers: &[(&String, &Decimal)]) -> Self {
        VotingPowerQuerier {
            powers: powers_to_map(powers),
        }
    }
}

pub(crate) fn powers_to_map(powers: &[(&String, &Decimal)]) -> HashMap<String, Decimal> {
    let mut powers_map: HashMap<String, Decimal> = HashMap::new();
    for (key, power) in powers.iter() {
        powers_map.insert(key.to_string(), (*power).clone());
    }
    powers_map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

#[derive(Clone, Default)]
pub struct ValkyrieConfigQuerier {
    campaign_deactivate_period: u64,
    reward_withdraw_burn_rate: Decimal,
}

impl ValkyrieConfigQuerier {
    pub fn new(
        campaign_deactivate_period: u64,
        reward_withdraw_burn_rate: Decimal,
    ) -> Self {
        ValkyrieConfigQuerier {
            campaign_deactivate_period,
            reward_withdraw_burn_rate,
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            voting_powers_querier: VotingPowerQuerier::default(),
            tax_querier: TaxQuerier::default(),
            valkyrie_config_querier: ValkyrieConfigQuerier::default(),
        }
    }

    pub fn with_voting_powers(&mut self, powers: &[(&String, &Decimal)]) {
        self.voting_powers_querier = VotingPowerQuerier::new(powers);
    }

    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    pub fn with_valkyrie_config(
        &mut self,
        campaign_deactivate_period: u64,
        reward_withdraw_burn_rate: Decimal,
    ) {
        self.valkyrie_config_querier = ValkyrieConfigQuerier::new(
            campaign_deactivate_period,
            reward_withdraw_burn_rate,
        );
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
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => {
                match from_binary(&msg) {
                    Ok(FactoryQueryMsg::CampaignConfig {}) => {
                        let res = CampaignConfigResponse {
                            reward_withdraw_burn_rate: self.valkyrie_config_querier.reward_withdraw_burn_rate,
                            campaign_deactivate_period: self.valkyrie_config_querier.campaign_deactivate_period,
                        };
                        SystemResult::Ok(ContractResult::from(to_binary(&res)))
                    },
                    _ => {
                        match from_binary(&msg) {
                            Ok(GovQueryMsg::VotingPower { address }) => {
                                let voting_power = match self.voting_powers_querier.powers.get(&address) {
                                    Some(v) => v.clone(),
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!("VotingPower is not found for {}", address),
                                            request: msg.clone(),
                                        })
                                    }
                                };

                                let res = VotingPowerResponse { voting_power };
                                SystemResult::Ok(ContractResult::from(to_binary(&res)))
                            },
                            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                                kind: msg.to_string(),
                            })
                        }
                    },
                }
            },
            _ => self.base.handle_query(request),
        }
    }
}
