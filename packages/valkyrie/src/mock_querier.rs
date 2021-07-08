use std::collections::HashMap;

use cosmwasm_std::{Api, Binary, CanonicalAddr, Coin, ContractResult, Decimal, from_slice, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, to_binary, Uint128, WasmQuery, from_binary, BankQuery, AllBalanceResponse, Addr, QuerierWrapper};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, MockApi, MockQuerier, MockStorage};
use cw20::TokenInfoResponse;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};
use crate::governance::query_msgs::{QueryMsg as GovQueryMsg, VotingPowerResponse, ContractConfigResponse as GovContractConfigResponse};
use crate::terra::calc_tax_one_plus;
use crate::campaign::query_msgs::{CampaignStateResponse, QueryMsg, BoosterResponse, ActiveBoosterResponse};
use crate::campaign_manager::query_msgs::{CampaignConfigResponse, BoosterConfigResponse, QueryMsg as CampaignManagerQueryMsg};
use crate::common::Denom;

pub type CustomDeps = OwnedDeps<MockStorage, MockApi, WasmMockQuerier>;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn custom_deps(contract_balance: &[Coin]) -> CustomDeps {
    let custom_querier = WasmMockQuerier::new(
        MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
    );

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_querier: TokenQuerier,
    tax_querier: TaxQuerier,
    voting_powers_querier: VotingPowerQuerier,
    campaign_manager_config_querier: CampaignManagerConfigQuerier,
    gov_config_querier: GovConfigQuerier,
    campaign_state_querier: CampaignStateQuerier,
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
pub struct CampaignManagerConfigQuerier {
    campaign_config: CampaignConfigResponse,
    booster_config: BoosterConfigResponse,
}

impl CampaignManagerConfigQuerier {
    pub fn new(
        campaign_config: CampaignConfigResponse,
        booster_config: BoosterConfigResponse,
    ) -> Self {
        CampaignManagerConfigQuerier {
            campaign_config,
            booster_config,
        }
    }
}

#[derive(Clone, Default)]
pub struct GovConfigQuerier {
    token_contract: String,
}

impl GovConfigQuerier {
    pub fn new(
        token_contract: String,
    ) -> Self {
        GovConfigQuerier {
            token_contract,
        }
    }
}

#[derive(Clone, Default)]
pub struct CampaignStateQuerier {
    states: HashMap<String, CampaignStateResponse>,
    active_boosters: HashMap<String, Option<BoosterResponse>>,
}

impl CampaignStateQuerier {
    pub fn new() -> Self {
        CampaignStateQuerier {
            states: HashMap::new(),
            active_boosters: HashMap::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&str, &[(&str, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&str, &[(&str, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&str, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&str, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(_) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request"),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(
                TerraQueryWrapper { route, query_data }
            ) => self.handle_custom(route, query_data),
            QueryRequest::Wasm(
                WasmQuery::Raw { contract_addr, key }
            ) => self.handle_wasm_raw(contract_addr, key),
            QueryRequest::Wasm(
                WasmQuery::Smart { contract_addr, msg }
            ) => self.handle_wasm_smart(contract_addr, msg),
            _ => self.base.handle_query(request),
        }
    }
    fn handle_custom(&self, route: &TerraRoute, query_data: &TerraQuery) -> QuerierResult {
        match route {
            TerraRoute::Treasury => self.handle_custom_treasury(query_data),
            _ => return QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_custom".to_string(),
            }),
        }
    }

    fn handle_custom_treasury(&self, query_data: &TerraQuery) -> QuerierResult {
        match query_data {
            TerraQuery::TaxRate {} => self.query_tax_rate(),
            TerraQuery::TaxCap { denom } => self.query_tax_cap(denom),
            _ => return QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_custom_treasury".to_string(),
            }),
        }
    }

    fn handle_wasm_raw(&self, contract_addr: &String, key: &Binary) -> QuerierResult {
        let key: &[u8] = key.as_slice();

        let mut result = self.query_token_info(contract_addr, key);

        if result.is_none() {
            result = self.query_balance(contract_addr, key);
        }

        if result.is_none() {
            return QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_raw".to_string(),
            });
        }

        result.unwrap()
    }

    fn handle_wasm_smart(&self, contract_addr: &String, msg: &Binary) -> QuerierResult {
        let mut result = self.handle_wasm_smart_campaign_manager(contract_addr, msg);

        if result.is_none() {
            result = self.handle_wasm_smart_governance(contract_addr, msg);
        }

        if result.is_none() {
            result = self.handle_wasm_smart_campaign(contract_addr, msg);
        }

        if result.is_none() {
            return QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart".to_string(),
            });
        }

        result.unwrap()
    }

    fn handle_wasm_smart_campaign_manager(&self, _contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(CampaignManagerQueryMsg::CampaignConfig {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &self.campaign_manager_config_querier.campaign_config,
                ))))
            },
            Ok(CampaignManagerQueryMsg::BoosterConfig {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &self.campaign_manager_config_querier.booster_config,
                ))))
            },
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn handle_wasm_smart_governance(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        if contract_addr != "GovernanceContract" {
            return None;
        }

        match from_binary(msg) {
            Ok(GovQueryMsg::ContractConfig {}) => {
                let response = GovContractConfigResponse {
                    governance_token: self.gov_config_querier.token_contract.clone(),
                };

                Some(SystemResult::Ok(ContractResult::from(to_binary(&response))))
            },
            Ok(GovQueryMsg::VotingPower { address }) => {
                let voting_power = match self.voting_powers_querier.powers.get(&address) {
                    Some(v) => v.clone(),
                    None => {
                        return Some(SystemResult::Err(SystemError::InvalidRequest {
                            error: format!("VotingPower is not found for {}", address),
                            request: msg.clone(),
                        }));
                    }
                };

                let response = VotingPowerResponse { voting_power };

                Some(SystemResult::Ok(ContractResult::from(to_binary(&response))))
            },
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn handle_wasm_smart_campaign(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(QueryMsg::CampaignState {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &self.campaign_state_querier.states[contract_addr],
                ))))
            },
            Ok(QueryMsg::ActiveBooster {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &ActiveBoosterResponse {
                        active_booster: self.campaign_state_querier.active_boosters[contract_addr].clone()
                    },
                ))))
            },
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn query_tax_rate(&self) -> QuerierResult {
        let response = TaxRateResponse {
            rate: self.tax_querier.rate,
        };

        SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
    }

    fn query_tax_cap(&self, denom: &String) -> QuerierResult {
        let response = TaxCapResponse {
            cap: self
                .tax_querier
                .caps
                .get(denom)
                .copied()
                .unwrap_or_default(),
        };

        SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
    }

    fn query_token_info(&self, contract_addr: &String, key: &[u8]) -> Option<QuerierResult> {
        if key.to_vec() != to_length_prefixed(b"token_info").to_vec() {
            return None;
        }

        let balances = self.token_querier.balances.get(contract_addr);

        if balances.is_none() {
            return Some(SystemResult::Err(SystemError::InvalidRequest {
                request: key.into(),
                error: format!(
                    "No balance info exists for the contract {}",
                    contract_addr,
                ),
            }));
        }

        let balances = balances.unwrap();

        let mut total_supply = Uint128::zero();

        for balance in balances {
            total_supply += *balance.1;
        }

        Some(SystemResult::Ok(ContractResult::Ok(
            to_binary(&TokenInfoResponse {
                name: format!("{}Token", contract_addr),
                symbol: format!("TOK"),
                decimals: 6,
                total_supply,
            }).unwrap(),
        )))
    }

    fn query_balance(&self, contract_addr: &String, key: &[u8]) -> Option<QuerierResult> {
        let prefix_balance = to_length_prefixed(b"balance").to_vec();
        if key[..prefix_balance.len()].to_vec() == prefix_balance {}

        let balances = self.token_querier.balances.get(contract_addr);

        if balances.is_none() {
            return Some(SystemResult::Err(SystemError::InvalidRequest {
                request: key.into(),
                error: format!(
                    "No balance info exists for the contract {}",
                    contract_addr,
                ),
            }));
        }

        let balances = balances.unwrap();

        let key_address: &[u8] = &key[prefix_balance.len()..];
        let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);
        let api = MockApi::default();
        let address = match api.addr_humanize(&address_raw) {
            Ok(v) => v.to_string(),
            Err(_) => {
                return Some(SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request"),
                    request: key.into(),
                }));
            }
        };
        let balance = match balances.get(&address) {
            Some(v) => v,
            None => {
                return Some(SystemResult::Err(SystemError::InvalidRequest {
                    error: "Balance not found".to_string(),
                    request: key.into(),
                }));
            }
        };

        Some(SystemResult::Ok(ContractResult::Ok(to_binary(&balance).unwrap())))
    }
}

const ZERO: Uint128 = Uint128::zero();

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            tax_querier: TaxQuerier::default(),
            campaign_manager_config_querier: CampaignManagerConfigQuerier::default(),
            voting_powers_querier: VotingPowerQuerier::default(),
            gov_config_querier: GovConfigQuerier::default(),
            campaign_state_querier: CampaignStateQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&str, &[(&str, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    pub fn with_global_campaign_config(
        &mut self,
        creation_fee_token: String,
        creation_fee_amount: Uint128,
        creation_fee_recipient: String,
        code_id: u64,
        distribution_denom_whitelist: Vec<Denom>,
        withdraw_fee_rate: Decimal,
        withdraw_fee_recipient: String,
        deactivate_period: u64,
    ) {
        self.campaign_manager_config_querier.campaign_config = CampaignConfigResponse {
            creation_fee_token,
            creation_fee_amount,
            creation_fee_recipient,
            code_id,
            distribution_denom_whitelist,
            withdraw_fee_rate,
            withdraw_fee_recipient,
            deactivate_period,
        };
    }

    pub fn with_gov_config(
        &mut self,
        token_contract: &str,
    ) {
        self.gov_config_querier = GovConfigQuerier::new(token_contract.to_string());
    }

    pub fn with_voting_powers(&mut self, powers: &[(&String, &Decimal)]) {
        self.voting_powers_querier = VotingPowerQuerier::new(powers);
    }

    pub fn with_booster_config(
        &mut self,
        booster_token: String,
        drop_booster_ratio: Decimal,
        activity_booster_ratio: Decimal,
        plus_booster_ratio: Decimal,
        activity_booster_multiplier: Decimal,
        min_participation_count: u64,
    ) {
        self.campaign_manager_config_querier.booster_config = BoosterConfigResponse {
            booster_token,
            drop_booster_ratio,
            activity_booster_ratio,
            plus_booster_ratio,
            activity_booster_multiplier,
            min_participation_count,
        }
    }

    pub fn with_campaign_state(
        &mut self,
        campaign: String,
        state: CampaignStateResponse,
    ) {
        self.campaign_state_querier.states.insert(campaign, state);
    }

    pub fn with_active_booster(
        &mut self,
        campaign: String,
        booster: Option<BoosterResponse>,
    ) {
        self.campaign_state_querier.active_boosters.insert(campaign, booster);
    }

    pub fn plus_token_balances(&mut self, balances: &[(&str, &[(&str, &Uint128)])]) {
        for (token_contract, balances) in balances.iter() {
            let token_contract = token_contract.to_string();

            if !self.token_querier.balances.contains_key(&token_contract) {
                self.token_querier.balances.insert(token_contract.clone(), HashMap::new());
            }
            let token_balances = self.token_querier.balances.get_mut(&token_contract).unwrap();

            for (account, balance) in balances.iter() {
                let account = account.to_string();
                let account_balance = token_balances.get(&account).unwrap_or(&ZERO);
                let new_balance = *account_balance + *balance;
                token_balances.insert(account, new_balance);
            }
        }
    }

    pub fn minus_token_balances(&mut self, balances: &[(&str, &[(&str, &Uint128)])]) {
        for (token_contract, balances) in balances.iter() {
            let token_contract = token_contract.to_string();

            if !self.token_querier.balances.contains_key(&token_contract) {
                self.token_querier.balances.insert(token_contract.clone(), HashMap::new());
            }
            let token_balances = self.token_querier.balances.get_mut(&token_contract).unwrap();

            for (account, balance) in balances.iter() {
                let account = account.to_string();
                let account_balance = token_balances.get(&account).unwrap_or(&ZERO);
                let new_balance = account_balance.checked_sub(**balance).unwrap();
                token_balances.insert(account, new_balance);
            }
        }
    }

    // configure the token owner mock querier
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&str, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    pub fn plus_native_balance(&mut self, address: &str, balances: Vec<Coin>) {
        let mut current_balances = from_binary::<AllBalanceResponse>(
            &self.base.handle_query(
                &QueryRequest::Bank(BankQuery::AllBalances { address: address.to_string() }),
            ).unwrap().unwrap()
        ).unwrap().amount;

        for coin in balances.iter() {
            let current_balance = current_balances.iter_mut()
                .find(|c| c.denom == coin.denom);

            if current_balance.is_some() {
                current_balance.unwrap().amount += coin.amount;
            } else {
                current_balances.push(coin.clone());
            }
        }

        self.base.update_balance(Addr::unchecked(address.to_string()), current_balances);
    }

    pub fn minus_native_balance(&mut self, address: &str, balances: Vec<Coin>) {
        let mut current_balances = from_binary::<AllBalanceResponse>(
            &self.base.handle_query(
                &QueryRequest::Bank(BankQuery::AllBalances { address: address.to_string() }),
            ).unwrap().unwrap()
        ).unwrap().amount;

        for coin in balances.iter() {
            let current_balance = current_balances.iter_mut()
                .find(|c| c.denom == coin.denom);

            if current_balance.is_some() {
                let coin_balance = current_balance.unwrap();
                coin_balance.amount = coin_balance.amount.checked_sub(coin.amount).unwrap();
            } else {
                panic!("Insufficient balance");
            }
        }

        self.base.update_balance(Addr::unchecked(address.to_string()), current_balances);
    }

    pub fn minus_native_balance_with_tax(&mut self, address: &str, mut balances: Vec<Coin>) {
        for balance in balances.iter_mut() {
            if balance.denom == "uluna" {
                return;
            }

            let tax = calc_tax_one_plus(
                &QuerierWrapper::new(self),
                balance.denom.clone(),
                balance.amount,
            ).unwrap();
            balance.amount = balance.amount + tax;
        }

        self.minus_native_balance(address, balances)
    }

    pub fn with_balance(&mut self, balances: &[(&str, &[Coin])]) {
        for (addr, balance) in balances {
            self.base.update_balance(Addr::unchecked(addr.to_string()), balance.to_vec());
        }
    }
}

// Copy from cosmwasm-storage v0.14.1
fn to_length_prefixed(namespace: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(namespace.len() + 2);
    out.extend_from_slice(&encode_length(namespace));
    out.extend_from_slice(namespace);
    out
}

// Copy from cosmwasm-storage v0.14.1
fn encode_length(namespace: &[u8]) -> [u8; 2] {
    if namespace.len() > 0xFFFF {
        panic!("only supports namespaces up to length 0xFFFF")
    }
    let length_bytes = (namespace.len() as u32).to_be_bytes();
    [length_bytes[2], length_bytes[3]]
}