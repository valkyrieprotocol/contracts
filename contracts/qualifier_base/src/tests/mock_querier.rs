use std::collections::HashMap;

use cosmwasm_std::*;
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, MockApi, MockQuerier, MockStorage};
use cw20::TokenInfoResponse;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};
use valkyrie::campaign::query_msgs::ActorResponse;

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
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_querier: TokenQuerier,
    tax_querier: TaxQuerier,
    campaign_querier: CampaignQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    #[allow(dead_code)]
    pub fn new(balances: &[(&str, &[(&str, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn new(rate: Decimal, caps: &[(&str, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

#[derive(Clone, Default)]
pub struct CampaignQuerier {
    // this lets us iterate over all pairs that match the first string
    actors: HashMap<String, ActorResponse>,
}

impl CampaignQuerier {
    #[allow(dead_code)]
    pub fn new(actors: HashMap<String, ActorResponse>) -> Self {
        CampaignQuerier {
            actors,
        }
    }
}

#[allow(dead_code)]
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
        let result = self.handle_wasm_smart_campaign(contract_addr, msg);

        if result.is_none() {
            return QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart".to_string(),
            });
        }

        result.unwrap()
    }

    fn handle_wasm_smart_campaign(&self, contract_addr: &String, msg: &Binary) -> Option<QuerierResult> {
        if contract_addr != "Campaign" {
            return None;
        }

        match from_binary(msg) {
            Ok(valkyrie::campaign::query_msgs::QueryMsg::Actor { address }) => {
                let default = ActorResponse::new(address.clone(), None);
                let actor = self.campaign_querier.actors.get(address.as_str())
                    .unwrap_or(&default);

                Some(SystemResult::Ok(ContractResult::from(to_binary(actor))))
            }
            Ok(_) => Some(QuerierResult::Err(SystemError::UnsupportedRequest {
                kind: "handle_wasm_smart:campaign".to_string(),
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
            campaign_querier: CampaignQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    #[allow(dead_code)]
    pub fn with_token_balances(&mut self, balances: &[(&str, &[(&str, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&str, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    pub fn plus_delegation(&mut self, delegator: &str, validator: &str, amount: Uint128) {
        let validators = from_binary::<AllValidatorsResponse>(
            &self.base.handle_query(
                &QueryRequest::Staking(StakingQuery::AllValidators {}),
            ).unwrap().unwrap()
        ).unwrap().validators;

        let mut delegations = from_binary::<AllDelegationsResponse>(
            &self.base.handle_query(
                &QueryRequest::Staking(StakingQuery::AllDelegations { delegator: delegator.to_string() }),
            ).unwrap().unwrap()
        ).unwrap().delegations;

        let delegation = delegations.iter_mut()
            .find(|d| d.delegator == Addr::unchecked(delegator) && d.validator == validator.to_string());

        match delegation {
            Some(delegation) => delegation.amount.amount += amount,
            None => delegations.push(Delegation {
                delegator: Addr::unchecked(delegator),
                validator: validator.to_string(),
                amount: coin(amount.u128(), "uluna")
            })
        }

        let delegations: Vec<FullDelegation> = delegations.iter().map(|d| FullDelegation {
            delegator: d.delegator.clone(),
            validator: d.validator.clone(),
            amount: d.amount.clone(),
            can_redelegate: d.amount.clone(),
            accumulated_rewards: vec![],
        }).collect();

        self.base.update_staking("uluna", validators.as_slice(), delegations.as_slice())
    }

    pub fn with_actor(&mut self, actor: ActorResponse) {
        self.campaign_querier.actors.insert(actor.address.clone(), actor);
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
