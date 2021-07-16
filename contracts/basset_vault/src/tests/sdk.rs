use crate::tests::mock_dependencies;
use crate::TOO_HIGH_BORROW_DEMAND_ERR_MSG;
use crate::{reply_response::MsgInstantiateContractResponse, SubmsgIds};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::StdResult;
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR},
    Api, CosmosMsg, Decimal, OwnedDeps, Querier, Reply, ReplyOn, Storage, SubMsg,
    SubMsgExecutionResponse, WasmMsg,
};
use cosmwasm_std::{to_binary, Coin, Empty, Response, Uint128};
use cw20::Cw20ReceiveMsg;
use cw20::MinterResponse;

use protobuf::Message;
use std::collections::HashMap;
use std::iter::FromIterator;
use yield_optimizer::basset_vault::YourselfMsg;

use yield_optimizer::basset_vault::Cw20HookMsg;
use yield_optimizer::basset_vault_strategy::BorrowerActionResponse;
use yield_optimizer::psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg;
use yield_optimizer::querier::{
    AnchorMarketEpochStateResponse, BorrowerInfoResponse, BorrowerResponse,
};
use yield_optimizer::{
    basset_vault::ExecuteMsg,
    nasset_token::InstantiateMsg as NAssetTokenInstantiateMsg,
    nasset_token_config_holder::{
        AnyoneMsg as NAssetTokenConfigHolderAnyoneMsg,
        ExecuteMsg as NAssetTokenConfigHolderExecuteMsg,
        InstantiateMsg as NAssetTokenConfigHolderInstantiateMsg,
    },
    nasset_token_rewards::InstantiateMsg as NAssetTokenRewardsInstantiateMsg,
};

use super::WasmMockQuerier;

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
pub const COLLATERAL_TOKEN_SYMBOL: &str = "Luna";
pub const BASSET_TOKEN_ADDR: &str = "addr0002";
pub const ANCHOR_CUSTODY_BASSET_CONTRACT: &str = "addr0003";
pub const ANCHOR_OVERSEER_CONTRACT: &str = "addr0004";
pub const ANCHOR_TOKEN: &str = "addr0006";
pub const ANC_STABLE_SWAP_CONTRACT: &str = "addr0008";
pub const PSI_STABLE_SWAP_CONTRACT: &str = "addr0009";
pub const BASSET_FARMER_STRATEGY_CONTRACT: &str = "addr0012";
pub const BASSET_FARMER_CONFIG_HOLDER_CONTRACT: &str = "addr0013";
pub const CLAIMING_REWARDS_DELAY: u64 = 1000;
pub const NASSET_TOKEN_CODE_ID: u64 = 10u64;
pub const NASSET_TOKEN_CONFIG_HOLDER_CODE_ID: u64 = 11u64;
pub const NASSET_TOKEN_REWARDS_CODE_ID: u64 = 12u64;
pub const PSI_DISTRIBUTOR_CODE_ID: u64 = 13u64;
pub const NASSET_TOKEN_HOLDERS_REWARDS_SHARE: u64 = 70;
pub const GOVERNANCE_STAKER_REWARDS_SHARE: u64 = 30;

pub struct Sdk {
    pub deps: OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    aterra_balance: Uint128,
    anc_balance: Uint128,
    basset_balance: Uint128,
    nasset_supply: Uint128,
    aterra_exchange_rate: Decimal256,
    borrower_action: BorrowerActionResponse,
}

impl Sdk {
    pub fn init() -> Self {
        let msg = yield_optimizer::basset_vault::InstantiateMsg {
            nasset_token_code_id: NASSET_TOKEN_CODE_ID,
            nasset_token_config_holder_code_id: NASSET_TOKEN_CONFIG_HOLDER_CODE_ID,
            nasset_token_rewards_code_id: NASSET_TOKEN_REWARDS_CODE_ID,
            psi_distributor_code_id: PSI_DISTRIBUTOR_CODE_ID,
            governance_contract_addr: GOVERNANCE_CONTRACT.to_string(),
            config_holder_addr: BASSET_FARMER_CONFIG_HOLDER_CONTRACT.to_string(),
            collateral_token_symbol: COLLATERAL_TOKEN_SYMBOL.to_string(),
            nasset_token_holders_psi_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
            governance_contract_psi_rewards_share: GOVERNANCE_STAKER_REWARDS_SHARE,
        };

        let mut deps = mock_dependencies(&[]);
        Self::instantiate_basset_vault(
            &mut deps,
            msg.clone(),
            &PSI_TOKEN,
            &NASSET_TOKEN_CONFIG_HOLDER_CONTRACT,
            &NASSET_TOKEN_ADDR,
            &NASSET_TOKEN_REWARDS_CONTRACT,
            &PSI_DISTRIBUTOR_CONTRACT,
        );

        Sdk {
            deps,
            aterra_balance: Uint128::zero(),
            anc_balance: Uint128::zero(),
            basset_balance: Uint128::zero(),
            nasset_supply: Uint128::zero(),
            aterra_exchange_rate: Decimal256::zero(),
            borrower_action: BorrowerActionResponse::Nothing,
        }
    }

    pub fn instantiate_basset_vault<A: Storage, B: Api, C: Querier>(
        deps: &mut OwnedDeps<A, B, C>,
        init_msg: yield_optimizer::basset_vault::InstantiateMsg,
        psi_token: &str,
        nasset_token_config_holder_contract: &str,
        nasset_contract_addr: &str,
        nasset_token_rewards_contract: &str,
        psi_distributor_contract: &str,
    ) {
        let info = mock_info("addr9999", &[]);

        // ==========================================================
        // ================ Instantiate BASSET_FARMER ===============
        // ==========================================================
        {
            let res =
                crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg.clone())
                    .unwrap();
            assert_eq!(
                res.messages,
                vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        code_id: init_msg.nasset_token_config_holder_code_id,
                        msg: to_binary(&NAssetTokenConfigHolderInstantiateMsg {
                            governance_contract_addr: init_msg.governance_contract_addr.clone()
                        })
                        .unwrap(),
                        funds: vec![],
                        label: "".to_string(),
                        admin: None,
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAssetConfigHolder.id(),
                    reply_on: ReplyOn::Success,
                }]
            );
        }

        // ==========================================================
        // ========= Instantiate NASSET_TOKEN_CONFIG_HOLDER =========
        // ==========================================================

        {
            let mut nasset_token_config_holder_initiate_response =
                MsgInstantiateContractResponse::new();
            nasset_token_config_holder_initiate_response
                .set_contract_address(nasset_token_config_holder_contract.to_string());

            let reply_msg = Reply {
                id: SubmsgIds::InitNAssetConfigHolder.id(),
                result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![],
                    data: Some(
                        nasset_token_config_holder_initiate_response
                            .write_to_bytes()
                            .unwrap()
                            .into(),
                    ),
                }),
            };

            let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
            assert_eq!(
                res.messages,
                vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        code_id: init_msg.nasset_token_code_id,
                        msg: to_binary(&NAssetTokenInstantiateMsg {
                            name: "nexus bAsset token share representation".to_string(),
                            symbol: format!("n{}", COLLATERAL_TOKEN_SYMBOL),
                            decimals: 6,
                            initial_balances: vec![],
                            mint: Some(MinterResponse {
                                minter: MOCK_CONTRACT_ADDR.to_string(),
                                cap: None,
                            }),
                            config_holder_contract: nasset_token_config_holder_contract.to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        label: "".to_string(),
                        admin: None,
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAsset.id(),
                    reply_on: ReplyOn::Success,
                }]
            );

            assert_eq!(
                res.attributes,
                vec![
                    attr("action", "nasset_token_config_holder_initialized"),
                    attr(
                        "nasset_token_config_holder_addr",
                        nasset_token_config_holder_contract,
                    ),
                ]
            );
        }

        // ==========================================================
        // ========= Instantiate NASSET_TOKEN =======================
        // ==========================================================
        {
            let mut nasset_token_initiate_response = MsgInstantiateContractResponse::new();
            nasset_token_initiate_response.set_contract_address(nasset_contract_addr.to_string());

            let reply_msg = Reply {
                id: SubmsgIds::InitNAsset.id(),
                result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![],
                    data: Some(
                        nasset_token_initiate_response
                            .write_to_bytes()
                            .unwrap()
                            .into(),
                    ),
                }),
            };

            let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
            assert_eq!(
                res.messages,
                vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        code_id: init_msg.nasset_token_rewards_code_id,
                        msg: to_binary(&NAssetTokenRewardsInstantiateMsg {
                            nasset_token_addr: nasset_contract_addr.to_string(),
                            psi_token_addr: PSI_TOKEN.to_string(),
                            governance_contract_addr: init_msg.governance_contract_addr.clone()
                        })
                        .unwrap(),
                        funds: vec![],
                        label: "".to_string(),
                        admin: None,
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAssetRewards.id(),
                    reply_on: ReplyOn::Success,
                }]
            );
            assert_eq!(
                res.attributes,
                vec![
                    attr("action", "nasset_token_initialized"),
                    attr("nasset_token_addr", nasset_contract_addr),
                ]
            );
        }

        // ==========================================================
        // ======== Set TOKEN_REWARDS_ADDR to CONFIG_HOLDER =========
        // ============= Instantiate PSI_DISTRIBUTOR ================
        // ==========================================================
        {
            let mut nasset_token_rewards_initiate_response = MsgInstantiateContractResponse::new();
            nasset_token_rewards_initiate_response
                .set_contract_address(nasset_token_rewards_contract.to_string());

            let reply_msg = Reply {
                id: SubmsgIds::InitNAssetRewards.id(),
                result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![],
                    data: Some(
                        nasset_token_rewards_initiate_response
                            .write_to_bytes()
                            .unwrap()
                            .into(),
                    ),
                }),
            };

            let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

            assert_eq!(
                res.messages,
                vec![
                    SubMsg {
                        msg: WasmMsg::Instantiate {
                            code_id: init_msg.psi_distributor_code_id,
                            msg: to_binary(&PsiDistributorInstantiateMsg {
                                psi_token_addr: psi_token.to_string(),
                                nasset_token_rewards_contract_addr: nasset_token_rewards_contract
                                    .to_string(),
                                nasset_token_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
                                governance_contract_addr: init_msg.governance_contract_addr.clone(),
                                governance_contract_share: GOVERNANCE_STAKER_REWARDS_SHARE,
                            })
                            .unwrap(),
                            funds: vec![],
                            label: "".to_string(),
                            admin: None,
                        }
                        .into(),
                        gas_limit: None,
                        id: SubmsgIds::InitPsiDistributor.id(),
                        reply_on: ReplyOn::Success,
                    },
                    SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: nasset_token_config_holder_contract.to_string(),
                        funds: vec![],
                        msg: to_binary(&NAssetTokenConfigHolderExecuteMsg::Anyone {
                            anyone_msg: NAssetTokenConfigHolderAnyoneMsg::SetTokenRewardsContract {
                                nasset_token_rewards_contract_addr: nasset_token_rewards_contract
                                    .to_string(),
                            },
                        })
                        .unwrap(),
                    }))
                ]
            );
            assert_eq!(
                res.attributes,
                vec![
                    attr("action", "nasset_token_rewards_initialized"),
                    attr("nasset_token_rewards_addr", nasset_token_rewards_contract),
                ]
            );
        }

        // ==========================================================
        // ============= PSI_DISTRIBUTOR initialized ================
        // ==========================================================
        {
            let mut psi_distributor_initiate_response = MsgInstantiateContractResponse::new();
            psi_distributor_initiate_response
                .set_contract_address(psi_distributor_contract.to_string());

            let reply_msg = Reply {
                id: SubmsgIds::InitPsiDistributor.id(),
                result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![],
                    data: Some(
                        psi_distributor_initiate_response
                            .write_to_bytes()
                            .unwrap()
                            .into(),
                    ),
                }),
            };

            let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
            assert!(res.messages.is_empty());
            assert_eq!(
                res.attributes,
                vec![
                    attr("action", "psi_distributor_initialized"),
                    attr("psi_distributor_addr", psi_distributor_contract),
                ]
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_loan(&mut self, value: Uint256) {
        self.deps.querier.with_loan(&[(
            &ANCHOR_MARKET_CONTRACT.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &BorrowerInfoResponse {
                    borrower: MOCK_CONTRACT_ADDR.to_string(),
                    loan_amount: value,
                    pending_rewards: Decimal256::zero(),
                },
            )],
        )]);
    }

    pub fn set_tax(&mut self, tax_percent: u64, cap: u128) {
        self.deps.querier.with_tax(
            Decimal::percent(tax_percent),
            &[(&STABLE_DENOM.to_string(), &Uint128::new(cap))],
        );
    }

    pub fn set_aterra_exchange_rate(&mut self, value: Decimal256) {
        self.aterra_exchange_rate = value;
        self.set_wasm_query_respones();
    }

    pub fn set_borrower_action(&mut self, value: BorrowerActionResponse) {
        self.borrower_action = value;
        self.set_wasm_query_respones();
    }

    fn set_wasm_query_respones(&mut self) {
        self.deps.querier.with_wasm_query_response(&[
            (
                &BASSET_FARMER_STRATEGY_CONTRACT.to_string(),
                &to_binary(&self.borrower_action).unwrap(),
            ),
            (
                &ANCHOR_MARKET_CONTRACT.to_string(),
                &to_binary(&AnchorMarketEpochStateResponse {
                    exchange_rate: self.aterra_exchange_rate,
                    aterra_supply: Uint256::from(1_000_000u64),
                })
                .unwrap(),
            ),
        ]);
    }

    pub fn set_collateral_balance(&mut self, balance: Uint256) {
        self.deps.querier.with_locked_basset(&[(
            &ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &BorrowerResponse { balance },
            )],
        )]);
    }

    pub fn set_aterra_balance(&mut self, value: Uint256) {
        self.aterra_balance = value.into();
        self.set_token_balances();
    }

    pub fn set_anc_balance(&mut self, value: Uint256) {
        self.anc_balance = value.into();
        self.set_token_balances();
    }

    pub fn set_nasset_supply(&mut self, value: Uint256) {
        self.nasset_supply = value.into();
        self.set_token_supplies();
    }

    pub fn set_basset_balance(&mut self, value: Uint256) {
        self.basset_balance = value.into();
        self.set_token_balances();
    }

    fn set_token_supplies(&mut self) {
        let supplies = vec![(NASSET_TOKEN_ADDR.to_string(), self.nasset_supply)];
        let supplies = HashMap::from_iter(supplies.into_iter());
        self.deps.querier.with_token_supplies(supplies)
    }

    fn set_token_balances(&mut self) {
        self.deps.querier.with_token_balances(&[
            (
                &ATERRA_TOKEN.to_string(),
                &[(&MOCK_CONTRACT_ADDR.to_string(), &self.aterra_balance)],
            ),
            (
                &BASSET_TOKEN_ADDR.to_string(),
                &[(&MOCK_CONTRACT_ADDR.to_string(), &self.basset_balance)],
            ),
            (
                &ANCHOR_TOKEN.to_string(),
                &[(&MOCK_CONTRACT_ADDR.to_string(), &self.anc_balance)],
            ),
        ]);
    }

    pub fn rebalance(&mut self) -> StdResult<Response<Empty>> {
        let rebalance_msg = yield_optimizer::basset_vault::AnyoneMsg::Rebalance;
        let info = mock_info(&"addr9999".to_string(), &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: rebalance_msg,
            },
        )
    }

    pub fn set_stable_balance(&mut self, value: Uint128) {
        self.deps.querier.update_base_balance(
            MOCK_CONTRACT_ADDR,
            vec![Coin {
                denom: STABLE_DENOM.to_string(),
                amount: value,
            }],
        );
    }

    pub fn aterra_redeem_success(&mut self) -> StdResult<Response<Empty>> {
        let reply_msg = Reply {
            id: SubmsgIds::RedeemStableOnRepayLoan.id(),
            result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                //we don't use it
                data: None,
            }),
        };

        crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg)
    }

    pub fn aterra_redeed_failed(&mut self) -> StdResult<Response<Empty>> {
        let reply_msg = Reply {
            id: SubmsgIds::RedeemStableOnRepayLoan.id(),
            result: cosmwasm_std::ContractResult::Err(format!(
                "fail to redeem aterra, cause: {}",
                TOO_HIGH_BORROW_DEMAND_ERR_MSG,
            )),
        };

        crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg)
    }

    pub fn continue_repay_loan(&mut self) -> StdResult<Response<Empty>> {
        let reply_msg = Reply {
            id: SubmsgIds::RepayLoan.id(),
            result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: None,
            }),
        };

        crate::contract::reply(self.deps.as_mut(), mock_env(), reply_msg)
    }

    pub fn user_deposit(&mut self, address: &str, amount: Uint128) -> StdResult<Response<Empty>> {
        let cw20_deposit_msg = Cw20ReceiveMsg {
            sender: address.to_string(),
            amount,
            msg: to_binary(&Cw20HookMsg::Deposit).unwrap(),
        };

        let info = mock_info(BASSET_TOKEN_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_deposit_msg),
        )
    }

    pub fn user_withdraw(&mut self, address: &str, amount: Uint128) -> StdResult<Response<Empty>> {
        let cw20_deposit_msg = Cw20ReceiveMsg {
            sender: address.to_string(),
            amount,
            msg: to_binary(&Cw20HookMsg::Withdraw).unwrap(),
        };

        let info = mock_info(NASSET_TOKEN_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_deposit_msg),
        )
    }

    pub fn user_send_honest_work(&mut self, block_height: u64) -> StdResult<Response<Empty>> {
        let honest_work_msg = yield_optimizer::basset_vault::AnyoneMsg::HonestWork;
        let mut env = mock_env();
        env.block.height = block_height;
        let info = mock_info(&"addr9999".to_string(), &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            env,
            info,
            ExecuteMsg::Anyone {
                anyone_msg: honest_work_msg,
            },
        )
    }

    pub fn send_swap_anc(&mut self) -> StdResult<Response<Empty>> {
        let info = mock_info(MOCK_CONTRACT_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Yourself {
                yourself_msg: YourselfMsg::SwapAnc,
            },
        )
    }

    pub fn send_distribute_rewards(&mut self) -> StdResult<Response<Empty>> {
        let info = mock_info(MOCK_CONTRACT_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Yourself {
                yourself_msg: YourselfMsg::DisributeRewards,
            },
        )
    }
}
