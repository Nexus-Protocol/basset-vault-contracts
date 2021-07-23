mod tests;

pub const MOCK_CONFIG_HOLDER_CONTRACT_ADDR: &str = "config_holder";
pub const MOCK_REWARDS_CONTRACT_ADDR: &str = "rewards";
pub const MOCK_TOKEN_CONTRACT_ADDR: &str = "token";
pub const MOCK_OWNER_ADDR: &str = "owner";

use basset_vault::nasset_token_config_holder::Config;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Addr, Coin, ContractResult, Decimal, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;

/// copypasted from TerraSwap
/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    let nasset_token_config = Config {
        nasset_token_rewards_contract: Addr::unchecked(MOCK_REWARDS_CONTRACT_ADDR),
        governance_contract: Addr::unchecked("whatever"),
    };
    custom_querier.with_nasset_token_config(nasset_token_config);

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    nasset_token_config: Option<Config>,
    base: MockQuerier<Empty>,
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
                if contract_addr != MOCK_CONFIG_HOLDER_CONTRACT_ADDR {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Wrong contract ({}) to ask for config", contract_addr),
                        request: key.into(),
                    });
                }

                let prefix_config = to_length_prefixed(b"config").to_vec();

                if key.to_vec() == prefix_config {
                    if let Some(ref nasset_token_config) = self.nasset_token_config {
                        SystemResult::Ok(ContractResult::from(to_binary(nasset_token_config)))
                    } else {
                        SystemResult::Err(SystemError::InvalidRequest {
                            error: format!("Nasset token config is not set"),
                            request: key.into(),
                        })
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            nasset_token_config: None,
            base,
        }
    }

    pub fn with_nasset_token_config(&mut self, config: Config) {
        self.nasset_token_config = Some(config);
    }
}
