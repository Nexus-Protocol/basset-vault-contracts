
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{OwnedDeps, Response, Uint128};
use yield_optimizer::psi_distributor::{AnyoneMsg, ExecuteMsg};

use crate::ContractResult;

use super::{mock_dependencies, WasmMockQuerier};

pub const PSI_TOKEN_ADDR: &str = "addr0001";
pub const NASSET_TOKEN_REWARDS_CONTRACT_ADDR: &str = "addr0002";
pub const GOVERNANCE_CONTRACT_ADDR: &str = "addr0003";
pub const NASSET_TOKEN_HOLDERS_REWARDS_SHARE: u64 = 70;
pub const GOVERNANCE_STAKER_REWARDS_SHARE: u64 = 30;

pub struct Sdk {
    pub deps: OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
}

impl Sdk {
    pub fn init() -> Self {
        let msg = yield_optimizer::psi_distributor::InstantiateMsg {
            psi_token_addr: PSI_TOKEN_ADDR.to_string(),
            nasset_token_rewards_contract_addr: NASSET_TOKEN_REWARDS_CONTRACT_ADDR.to_string(),
            nasset_token_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
            governance_contract_addr: GOVERNANCE_CONTRACT_ADDR.to_string(),
            governance_contract_share: GOVERNANCE_STAKER_REWARDS_SHARE,
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
            anyone_msg: AnyoneMsg::DistributeRewards,
        };

        let info = mock_info("addr9999", &[]);
        crate::contract::execute(self.deps.as_mut(), mock_env(), info, distribute_msg)
    }
}
