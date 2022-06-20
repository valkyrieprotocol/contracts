use std::collections::HashMap;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use cosmwasm_std::{Api, Binary, CanonicalAddr, Coin, ContractResult, Decimal, from_slice, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, to_binary, Uint128, WasmQuery, from_binary, BankQuery, AllBalanceResponse, Addr, CustomQuery};
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, MockApi, MockQuerier, MockStorage};
use cw20::{TokenInfoResponse, Cw20QueryMsg};
use crate::governance::query_msgs::{QueryMsg as GovQueryMsg, VotingPowerResponse, ContractConfigResponse as GovContractConfigResponse, StakerStateResponse};
use crate::campaign::query_msgs::{CampaignStateResponse, QueryMsg};
use crate::campaign_manager::query_msgs::{QueryMsg as CampaignManagerQueryMsg, ConfigResponse, ReferralRewardLimitOptionResponse};
use crate::proxy::execute_msgs::SwapOperation;

use crate::proxy::query_msgs::SimulateSwapOperationsResponse;
use crate::proxy::query_msgs::QueryMsg::SimulateSwapOperations;
use crate::test_constants::campaign_manager::CAMPAIGN_MANAGER;
use crate::test_constants::governance::GOVERNANCE;
use crate::test_constants::VALKYRIE_PROXY;

pub type CustomDeps = OwnedDeps<MockStorage, MockApi, WasmMockQuerier>;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn custom_deps() -> CustomDeps {
    let custom_querier = WasmMockQuerier::new(
        MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]),
    );

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<QueryWrapper>,
    token_querier: TokenQuerier,
    voting_powers_querier: VotingPowerQuerier,
    campaign_manager_config_querier: CampaignManagerConfigQuerier,
    governance_querier: GovConfigQuerier,
    campaign_state_querier: CampaignStateQuerier,
    astroport_router_querier: AstroportRouterQuerier,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryWrapper {}

// implement custom query
impl CustomQuery for QueryWrapper {}

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
    config: ConfigResponse,
    referral_reward_limit_option: ReferralRewardLimitOptionResponse,
}

impl CampaignManagerConfigQuerier {
    pub fn new(
        config: ConfigResponse,
        referral_reward_limit_option: ReferralRewardLimitOptionResponse,
    ) -> Self {
        CampaignManagerConfigQuerier {
            config,
            referral_reward_limit_option,
        }
    }
}

#[derive(Clone, Default)]
pub struct GovConfigQuerier {
    token_contract: String,
    staker_state: HashMap<String, StakerStateResponse>,
}

impl GovConfigQuerier {
    pub fn new(
        token_contract: String,
        staker_state: HashMap<String, StakerStateResponse>,
    ) -> Self {
        GovConfigQuerier {
            token_contract,
            staker_state,
        }
    }
}

#[derive(Clone, Default)]
pub struct CampaignStateQuerier {
    states: HashMap<String, CampaignStateResponse>,
}

impl CampaignStateQuerier {
    pub fn new() -> Self {
        CampaignStateQuerier {
            states: HashMap::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct AstroportRouterQuerier {
    prices: HashMap<(String, String), f64>,
}

impl AstroportRouterQuerier {
    pub fn new() -> Self {
        AstroportRouterQuerier {
            prices: HashMap::new(),
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

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<QueryWrapper> = match from_slice(bin_request) {
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
    pub fn handle_query(&self, request: &QueryRequest<QueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(
                WasmQuery::Raw { contract_addr, key }
            ) => self.handle_wasm_raw(contract_addr, key),
            QueryRequest::Wasm(
                WasmQuery::Smart { contract_addr, msg }
            ) => self.handle_wasm_smart(contract_addr, msg),
            _ => self.base.handle_query(request),
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
            result = self.handle_wasm_smart_astroport_router(contract_addr, msg);
        }

        if result.is_none() {
            result = self.handle_cw20(contract_addr, msg);
        }

        if result.is_none() {
            return QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart".to_string(),
            });
        }

        result.unwrap()
    }

    fn handle_wasm_smart_campaign_manager(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        if contract_addr != CAMPAIGN_MANAGER {
            return None;
        }

        match from_binary(msg) {
            Ok(CampaignManagerQueryMsg::Config {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &self.campaign_manager_config_querier.config,
                ))))
            }
            Ok(CampaignManagerQueryMsg::ReferralRewardLimitOption {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &self.campaign_manager_config_querier.referral_reward_limit_option,
                ))))
            }
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart:campaign_manager".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn handle_wasm_smart_governance(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        if contract_addr != GOVERNANCE {
            return None;
        }

        match from_binary(msg) {
            Ok(GovQueryMsg::ContractConfig {}) => {
                let response = GovContractConfigResponse {
                    governance_token: self.governance_querier.token_contract.clone(),
                };

                Some(SystemResult::Ok(ContractResult::from(to_binary(&response))))
            }
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
            }
            Ok(GovQueryMsg::StakerState { address }) => {
                let default = StakerStateResponse::default();
                let response = self.governance_querier.staker_state
                    .get(address.as_str())
                    .unwrap_or(&default);

                Some(SystemResult::Ok(ContractResult::from(to_binary(response))))
            }
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart:governance".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn handle_wasm_smart_campaign(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        if !self.campaign_state_querier.states.contains_key(contract_addr) {
            return None;
        }

        match from_binary(msg) {
            Ok(QueryMsg::CampaignState {}) => {
                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &self.campaign_state_querier.states[contract_addr],
                ))))
            }
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart:campaign".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn handle_wasm_smart_astroport_router(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        if contract_addr != VALKYRIE_PROXY {
            return None;
        }

        match from_binary(msg) {
            Ok(SimulateSwapOperations { offer_amount, operations }) => {
                let mut amount = offer_amount.u128();
                for operation in operations.iter() {
                    let price = self.astroport_router_querier.prices
                        .get(&swap_operation_to_string(operation))
                        .unwrap();

                    amount = (amount as f64 * *price) as u128;
                }

                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &SimulateSwapOperationsResponse {
                        amount: Uint128::new(amount),
                    }
                ))))
            }
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart:valkyrie_proxy".to_string(),
            })),
            Err(_) => None,
        }
    }

    fn handle_cw20(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        match from_binary(msg) {
            Ok(Cw20QueryMsg::Balance { address }) => {
                let default = Uint128::zero();
                let balance = self.token_querier.balances[contract_addr].get(address.as_str())
                    .unwrap_or(&default).clone();

                Some(SystemResult::Ok(ContractResult::from(to_binary(
                    &cw20::BalanceResponse { balance },
                ))))
            },
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart:cw20".to_string(),
            })),
            Err(_) => None,
        }
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
    pub fn new(base: MockQuerier<QueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            campaign_manager_config_querier: CampaignManagerConfigQuerier::default(),
            voting_powers_querier: VotingPowerQuerier::default(),
            governance_querier: GovConfigQuerier::default(),
            campaign_state_querier: CampaignStateQuerier::default(),
            astroport_router_querier: AstroportRouterQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&str, &[(&str, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    pub fn with_global_campaign_config(
        &mut self,
        config: ConfigResponse,
    ) {
        self.campaign_manager_config_querier.config = config;
    }

    pub fn with_referral_reward_limit_option(
        &mut self,
        option: ReferralRewardLimitOptionResponse,
    ) {
        self.campaign_manager_config_querier.referral_reward_limit_option = option;
    }

    pub fn with_gov_config(
        &mut self,
        token_contract: &str,
    ) {
        self.governance_querier.token_contract = token_contract.to_string();
    }

    pub fn with_gov_staker_state(
        &mut self,
        address: &str,
        state: StakerStateResponse,
    ) {
        self.governance_querier.staker_state.insert(address.to_string(), state);
    }

    pub fn with_voting_powers(&mut self, powers: &[(&String, &Decimal)]) {
        self.voting_powers_querier = VotingPowerQuerier::new(powers);
    }

    pub fn with_campaign_state(
        &mut self,
        campaign: String,
        state: CampaignStateResponse,
    ) {
        self.campaign_state_querier.states.insert(campaign, state);
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

    pub fn with_balance(&mut self, balances: &[(&str, &[Coin])]) {
        for (addr, balance) in balances {
            self.base.update_balance(Addr::unchecked(addr.to_string()), balance.to_vec());
        }
    }

    pub fn with_astroport_price(&mut self, offer: String, ask: String, price: f64) {
        self.astroport_router_querier.prices.insert((offer, ask), price);
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

fn swap_operation_to_string(operation: &SwapOperation) -> (String, String) {
    match operation {
        SwapOperation::Swap { offer_asset_info, ask_asset_info } => {
            (offer_asset_info.to_string(), ask_asset_info.to_string())
        }
    }
}