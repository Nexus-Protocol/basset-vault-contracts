use basset_vault::psi_distributor::{AnyoneMsg, ExecuteMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{OwnedDeps, Response, Uint128};

use super::{mock_dependencies, WasmMockQuerier};
use crate::ContractResult;
use std::str::FromStr;

pub const PSI_TOKEN_ADDR: &str = "addr0001";
pub const NASSET_TOKEN_REWARDS_CONTRACT_ADDR: &str = "addr0002";
pub const GOVERNANCE_CONTRACT_ADDR: &str = "addr0003";
pub const COMMUNITY_POOL_CONTRACT_ADDR: &str = "addr0004";
pub const BASSET_VAULT_STRATEGY_CONTRACT_ADDR: &str = "addr0005";
pub const NASSET_PSI_SWAP_CONTRACT_ADDR: &str = "addr0006";
pub const AIM_LTV: &str = "0.8";
pub const MANUAL_LTV: &str = "0.6";
pub const FEE_RATE: &str = "0.5";
pub const TAX_RATE: &str = "0.25";

pub struct Sdk {
    pub deps: OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
}

impl Sdk {
    pub fn init() -> Self {
        let msg = basset_vault::psi_distributor::InstantiateMsg {
            psi_token_addr: PSI_TOKEN_ADDR.to_string(),
            governance_contract_addr: GOVERNANCE_CONTRACT_ADDR.to_string(),
            nasset_token_rewards_contract_addr: NASSET_TOKEN_REWARDS_CONTRACT_ADDR.to_string(),
            community_pool_contract_addr: COMMUNITY_POOL_CONTRACT_ADDR.to_string(),
            basset_vault_strategy_contract_addr: BASSET_VAULT_STRATEGY_CONTRACT_ADDR.to_string(),
            nasset_psi_swap_contract_addr: NASSET_PSI_SWAP_CONTRACT_ADDR.to_string(),
            manual_ltv: Decimal256::from_str(MANUAL_LTV).unwrap(),
            fee_rate: Decimal256::from_str(FEE_RATE).unwrap(),
            tax_rate: Decimal256::from_str(TAX_RATE).unwrap(),
        };

        let mut deps = mock_dependencies(&[]);

        let env = mock_env();
        let info = mock_info("addr9999", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

        Sdk { deps }
    }

    pub fn set_psi_balance(&mut self, value: Uint128) {
        self.deps.querier.with_token_balances(&[(
            &PSI_TOKEN_ADDR.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &value)],
        )]);
    }

    pub fn distribute_rewards(&mut self) -> ContractResult<Response> {
        let distribute_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::DistributeRewards {},
        };

        let info = mock_info("addr9999", &[]);
        crate::contract::execute(self.deps.as_mut(), mock_env(), info, distribute_msg)
    }
}
