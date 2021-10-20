use crate::ContractResult;
use crate::{contract::query, tests::mock_dependencies};

use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR},
    Addr, Decimal, OwnedDeps,
};
use cosmwasm_std::{Empty, Response, Uint128};

use basset_vault::nasset_token_rewards::{ExecuteMsg, HolderResponse, QueryMsg};

use super::WasmMockQuerier;

pub const PSI_TOKEN_ADDR: &str = "addr0001";
pub const NASSET_TOKEN_ADDR: &str = "addr0002";
pub const GOVERNANCE_CONTRACT_ADDR: &str = "addr0003";

pub struct Sdk {
    pub deps: OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
}

impl Sdk {
    pub fn init() -> Self {
        let msg = basset_vault::nasset_token_rewards::InstantiateMsg {
            psi_token_addr: PSI_TOKEN_ADDR.to_string(),
            nasset_token_addr: NASSET_TOKEN_ADDR.to_string(),
            governance_contract_addr: GOVERNANCE_CONTRACT_ADDR.to_string(),
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

    pub fn increase_user_balance(&mut self, user_addr: &Addr, deposit_amount: Uint128) {
        let user_increase_balance = basset_vault::nasset_token_rewards::TokenMsg::IncreaseBalance {
            address: user_addr.to_string(),
            amount: deposit_amount,
        };

        let info = mock_info(NASSET_TOKEN_ADDR, &vec![]);
        let res = crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: user_increase_balance,
            },
        );
        assert!(res.is_ok());
    }

    pub fn decrease_user_balance(&mut self, user_addr: &Addr, withdraw_amount: Uint128) {
        let decrease_balance_msg = basset_vault::nasset_token_rewards::TokenMsg::DecreaseBalance {
            address: user_addr.to_string(),
            amount: withdraw_amount,
        };

        let info = mock_info(NASSET_TOKEN_ADDR, &vec![]);
        let res = crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Token {
                token_msg: decrease_balance_msg,
            },
        );
        assert!(res.is_ok());
    }

    pub fn update_index(&mut self) -> ContractResult<Response> {
        let update_index_msg = basset_vault::nasset_token_rewards::AnyoneMsg::UpdateGlobalIndex {};
        let info = mock_info(&"addr9999".to_string(), &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: update_index_msg,
            },
        )
    }

    pub fn claim_rewards(&mut self, sender: &Addr) -> ContractResult<Response<Empty>> {
        let claim_msg =
            basset_vault::nasset_token_rewards::AnyoneMsg::ClaimRewards { recipient: None };
        let info = mock_info(&sender.to_string(), &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: claim_msg,
            },
        )
    }

    pub fn query_holder_state(
        &self,
        holder: &Addr,
        expected_balance: Uint128,
        expected_index: Decimal,
        expected_pending_rewards: Decimal,
    ) {
        let holder_query_res = query(
            self.deps.as_ref(),
            mock_env(),
            QueryMsg::Holder {
                address: holder.to_string(),
            },
        )
        .unwrap();

        let query_response: HolderResponse = from_binary(&holder_query_res).unwrap();
        assert_eq!(
            query_response,
            HolderResponse {
                address: holder.to_string(),
                balance: expected_balance,
                index: expected_index,
                pending_rewards: expected_pending_rewards,
            }
        );
    }
}
