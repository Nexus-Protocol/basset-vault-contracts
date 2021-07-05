mod deposit_basset;
mod instantiate;
mod repay_loan;
mod sdk;

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Api, Binary, CanonicalAddr, Coin, ContractResult, Decimal,
    OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;
use std::hash::Hash;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};
use yield_optimizer::querier::{BorrowerInfoResponse, BorrowerResponse};

use cw20::TokenInfoResponse;

/// copypasted from TerraSwap
/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    borrowers_info: HashMap<String, HashMap<String, BorrowerInfoResponse>>,
    borrowers: HashMap<String, HashMap<String, BorrowerResponse>>,
    base: MockQuerier<TerraQueryWrapper>,
    tax_querier: TaxQuerier,
    token_querier: TokenQuerier,
    wasm_query_smart_responses: HashMap<String, Binary>,
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
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
    supplies: HashMap<String, Uint128>,
}

pub(crate) fn array_to_hashmap<K, V>(
    balances: &[(&String, &[(&K, &V)])],
) -> HashMap<String, HashMap<K, V>>
where
    V: Clone,
    K: Clone + Eq + Hash,
{
    let mut result_map: HashMap<String, HashMap<K, V>> = HashMap::new();
    for (contract_addr, map_values) in balances.iter() {
        let mut contract_balances_map: HashMap<K, V> = HashMap::new();
        for (key, value) in map_values.iter() {
            contract_balances_map.insert((**key).clone(), (**value).clone());
        }

        result_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    result_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
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
                if &TerraRoute::Treasury == route {
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

            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();

                if let Some(borrower_res) = self.try_get_borrower(key, contract_addr) {
                    return borrower_res;
                }
                if let Some(borrower_info_res) = self.try_get_borrower_info(key, contract_addr) {
                    return borrower_info_res;
                }

                let prefix_token_info = to_length_prefixed(b"token_info").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                if key.to_vec() == prefix_token_info {
                    let token_supply = match self.token_querier.supplies.get(contract_addr) {
                        Some(supply) => supply,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!(
                                    "No supply info exists for the contract {}",
                                    contract_addr
                                ),
                                request: key.into(),
                            })
                        }
                    };

                    SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                        name: "some_token_name".to_string(),
                        symbol: "some_token_symbol".to_string(),
                        decimals: 6,
                        total_supply: *token_supply,
                    })))
                } else if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key[prefix_balance.len()..];
                    let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);

                    let api: MockApi = MockApi::default();
                    let address: Addr = match api.addr_humanize(&address_raw) {
                        Ok(v) => v,
                        Err(e) => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!("Parsing query request: {}", e),
                                request: key.into(),
                            })
                        }
                    };

                    let balances: &HashMap<String, Uint128> =
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances) => balances,
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!(
                                        "No balance info exists for the contract {}",
                                        contract_addr
                                    ),
                                    request: key.into(),
                                })
                            }
                        };

                    let balance = match balances.get(&address.to_string()) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            })
                        }
                    };

                    SystemResult::Ok(ContractResult::from(to_binary(&balance)))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                let response = match self.wasm_query_smart_responses.get(contract_addr) {
                    Some(response) => response,
                    None => {
                        return SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No WasmQuery::Smart responses exists for the contract {}",
                                contract_addr
                            ),
                            request: (*msg).clone(),
                        })
                    }
                };

                SystemResult::Ok(ContractResult::Ok(response.clone()))
            }
            _ => self.base.handle_query(request),
        }
    }

    fn try_get_borrower(&self, key: &[u8], contract_addr: &String) -> Option<QuerierResult> {
        let prefix_borrower_info = to_length_prefixed(b"borrower").to_vec();
        if key[..prefix_borrower_info.len()].to_vec() == prefix_borrower_info {
            let key_address: &[u8] = &key[prefix_borrower_info.len()..];
            let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);

            let api: MockApi = MockApi::default();
            let address: Addr = match api.addr_humanize(&address_raw) {
                Ok(v) => v,
                Err(e) => {
                    return Some(SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: key.into(),
                    }))
                }
            };

            let empty_borrowers = HashMap::new();
            let default_borrower = BorrowerResponse {
                borrower: address.to_string(),
                balance: Uint256::zero(),
                spendable: Uint256::zero(),
            };

            let borrowers_map = self
                .borrowers
                .get(contract_addr)
                .unwrap_or(&empty_borrowers);
            let borrower = borrowers_map
                .get(&address.to_string())
                .unwrap_or(&default_borrower);

            return Some(SystemResult::Ok(ContractResult::from(to_binary(&borrower))));
        } else {
            return None;
        }
    }

    fn try_get_borrower_info(&self, key: &[u8], contract_addr: &String) -> Option<QuerierResult> {
        let prefix_borrower_info = to_length_prefixed(b"liability").to_vec();
        if key[..prefix_borrower_info.len()].to_vec() == prefix_borrower_info {
            let key_address: &[u8] = &key[prefix_borrower_info.len()..];
            let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);

            let api: MockApi = MockApi::default();
            let address: Addr = match api.addr_humanize(&address_raw) {
                Ok(v) => v,
                Err(e) => {
                    return Some(SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: key.into(),
                    }))
                }
            };

            let empty_borrowers_info = HashMap::new();
            let default_borrower_info = BorrowerInfoResponse {
                borrower: address.to_string(),
                interest_index: Decimal256::one(),
                reward_index: Decimal256::zero(),
                loan_amount: Uint256::zero(),
                pending_rewards: Decimal256::zero(),
            };

            let borrowers_info_map = self
                .borrowers_info
                .get(contract_addr)
                .unwrap_or(&empty_borrowers_info);
            let borrower_info = borrowers_info_map
                .get(&address.to_string())
                .unwrap_or(&default_borrower_info);

            return Some(SystemResult::Ok(ContractResult::from(to_binary(
                &borrower_info,
            ))));
        } else {
            return None;
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            borrowers_info: HashMap::new(),
            borrowers: HashMap::new(),
            base,
            token_querier: TokenQuerier::default(),
            wasm_query_smart_responses: HashMap::new(),
            tax_querier: TaxQuerier::default(),
        }
    }

    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier.balances = array_to_hashmap(balances);
    }

    pub fn with_token_supplies(&mut self, supplies: HashMap<String, Uint128>) {
        self.token_querier.supplies = supplies;
    }

    pub fn with_loan(&mut self, borrowers_info: &[(&String, &[(&String, &BorrowerInfoResponse)])]) {
        self.borrowers_info = array_to_hashmap(borrowers_info);
    }

    pub fn with_wasm_query_response(&mut self, contract_responses_map: &[(&String, &Binary)]) {
        let mut result_map: HashMap<String, Binary> = HashMap::new();
        for (contract_addr, response) in contract_responses_map.iter() {
            result_map.insert((**contract_addr).clone(), (**response).clone());
        }
        self.wasm_query_smart_responses = result_map;
    }
    //
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    pub fn update_base_balance(&mut self, addr: &str, balance: Vec<Coin>) -> Option<Vec<Coin>> {
        self.base.update_balance(addr, balance)
    }
}
