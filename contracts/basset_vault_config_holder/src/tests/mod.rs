mod change_config;
mod instantiate;

use basset_vault::querier::BorrowerResponse;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Api, CanonicalAddr, Coin, ContractResult, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;

use cw20::TokenInfoResponse;

pub const NASSET_TOKEN_ADDR: &str = "addr0001";
pub const ATERRA_TOKEN: &str = "addr0010";
pub const STABLE_DENOM: &str = "uust";
pub const ANCHOR_MARKET_CONTRACT: &str = "addr0007";
pub const PSI_DISTRIBUTOR_CONTRACT: &str = "addr0015";
pub const GOVERNANCE_CONTRACT: &str = "addr0016";
pub const PSI_TOKEN: &str = "addr0011";
pub const NASSET_TOKEN_CONFIG_HOLDER_CONTRACT: &str = "addr0017";
pub const NASSET_TOKEN_REWARDS_CONTRACT: &str = "addr0018";
pub const OVER_LOAN_BALANCE_VALUE: &str = "1.01";
pub const BASSET_TOKEN_ADDR: &str = "addr0002";
pub const ANCHOR_CUSTODY_BASSET_CONTRACT: &str = "addr0003";
pub const ANCHOR_OVERSEER_CONTRACT: &str = "addr0004";
pub const ANCHOR_TOKEN: &str = "addr0006";
pub const ANC_STABLE_SWAP_CONTRACT: &str = "addr0008";
pub const PSI_STABLE_SWAP_CONTRACT: &str = "addr0009";
pub const BASSET_FARMER_CONFIG_CONTRACT: &str = "addr0012";
pub const CLAIMING_REWARDS_DELAY: u64 = 1000;
pub const NASSET_TOKEN_CODE_ID: u64 = 10u64;
pub const NASSET_TOKEN_CONFIG_HOLDER_CODE_ID: u64 = 11u64;
pub const NASSET_TOKEN_REWARDS_CODE_ID: u64 = 12u64;
pub const PSI_DISTRIBUTOR_CODE_ID: u64 = 13u64;
pub const NASSET_TOKEN_HOLDERS_REWARDS_SHARE: u64 = 70;
pub const GOVERNANCE_STAKER_REWARDS_SHARE: u64 = 30;

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
    borrowers_info: HashMap<String, HashMap<String, BorrowerResponse>>,
    base: MockQuerier<Empty>,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
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
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
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
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();

                let prefix_borrower_info = to_length_prefixed(b"borrower").to_vec();
                if key[..prefix_borrower_info.len()].to_vec() == prefix_borrower_info {
                    let key_address: &[u8] = &key[prefix_borrower_info.len()..];
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

                    let empty_borrowers = HashMap::new();
                    let default_borrower = BorrowerResponse {
                        balance: Uint256::zero(),
                    };

                    let borrowers_map = self
                        .borrowers_info
                        .get(contract_addr)
                        .unwrap_or(&empty_borrowers);
                    let borrower_info = borrowers_map
                        .get(&address.to_string())
                        .unwrap_or(&default_borrower);

                    return SystemResult::Ok(ContractResult::from(to_binary(&borrower_info)));
                }

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

                let prefix_token_info = to_length_prefixed(b"token_info").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                if key.to_vec() == prefix_token_info {
                    let mut total_supply = Uint128::zero();

                    for balance in balances {
                        total_supply += *balance.1;
                    }

                    SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                        name: "some_token_name".to_string(),
                        symbol: "some_token_symbol".to_string(),
                        decimals: 6,
                        total_supply,
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
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier) -> Self {
        WasmMockQuerier {
            borrowers_info: HashMap::new(),
            base,
            token_querier: TokenQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
