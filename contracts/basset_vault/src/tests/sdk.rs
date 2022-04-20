use crate::tests::mock_dependencies;
use crate::TOO_HIGH_BORROW_DEMAND_ERR_MSG;
use crate::{reply_response::MsgInstantiateContractResponse, SubmsgIds};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR},
    Api, CosmosMsg, Decimal, OwnedDeps, Querier, Reply, ReplyOn, Storage, SubMsg,
    SubMsgExecutionResponse, WasmMsg,
};
use cosmwasm_std::{to_binary, Coin, Empty, Event, Response, Uint128};
use cosmwasm_std::{Addr, StdResult};
use cw20::Cw20ReceiveMsg;
use cw20::MinterResponse;

use basset_vault::basset_vault::YourselfMsg;
use protobuf::Message;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::str::FromStr;

use basset_vault::anchor::basset_custody::BorrowerInfo as AnchorBassetCustodyBorrowerInfo;
use basset_vault::anchor::market::BorrowerInfoResponse as AnchorMarketBorrowerInfo;
use basset_vault::astroport_factory::{ExecuteMsg as AstroportFactoryExecuteMsg, PairType};
use basset_vault::basset_vault::Cw20HookMsg;
use basset_vault::basset_vault_strategy::BorrowerActionResponse;
use basset_vault::psi_distributor::InstantiateMsg as PsiDistributorInstantiateMsg;
use basset_vault::querier::{AnchorMarketEpochStateResponse, AnchorMarketQueryMsg};
use basset_vault::terraswap::AssetInfo;
use basset_vault::{
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
pub const STABLE_DENOM: &str = "uusd";
pub const ANCHOR_MARKET_CONTRACT: &str = "addr0007";
pub const PSI_DISTRIBUTOR_CONTRACT: &str = "addr0015";
pub const GOVERNANCE_CONTRACT: &str = "addr0016";
pub const PSI_TOKEN: &str = "addr0011";
pub const NASSET_TOKEN_CONFIG_HOLDER_CONTRACT: &str = "addr0017";
pub const NASSET_TOKEN_REWARDS_CONTRACT: &str = "addr0018";
pub const COLLATERAL_TOKEN_SYMBOL: &str = "Luna";
pub const BASSET_TOKEN_ADDR: &str = "addr0002";
pub const ANCHOR_CUSTODY_BASSET_CONTRACT: &str = "addr0003";
pub const ANCHOR_OVERSEER_CONTRACT: &str = "addr0004";
pub const ANCHOR_TOKEN: &str = "addr0006";
pub const ANC_STABLE_SWAP_CONTRACT: &str = "addr0008";
pub const PSI_STABLE_SWAP_CONTRACT: &str = "addr0009";
pub const ASTROPORT_FACTORY_CONTRACT_ADDR: &str = "addr0014";
pub const BASSET_VAULT_STRATEGY_CONTRACT: &str = "addr0012";
pub const COMMUNITY_POOL_CONTRACT_ADDR: &str = "addr0013";
pub const NASSET_PSI_SWAP_CONTRACT_ADDR: &str = "addr0019";
pub const ANCHOR_BASSET_REWARD_CONTRACT: &str = "addr0020";
pub const CLAIMING_REWARDS_DELAY: u64 = 1000;
pub const NASSET_TOKEN_CODE_ID: u64 = 10u64;
pub const NASSET_TOKEN_CONFIG_HOLDER_CODE_ID: u64 = 11u64;
pub const NASSET_TOKEN_REWARDS_CODE_ID: u64 = 12u64;
pub const PSI_DISTRIBUTOR_CODE_ID: u64 = 13u64;
pub const OVER_LOAN_BALANCE_VALUE: &str = "1.01";
pub const MANUAL_LTV: &str = "0.6";
pub const FEE_RATE: &str = "0.5";
pub const TAX_RATE: &str = "0.25";

pub struct Sdk {
    pub deps: OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    aterra_balance: Uint128,
    anc_balance: Uint128,
    basset_balance: Uint128,
    nasset_supply: Uint128,
    aterra_exchange_rate: Decimal256,
    anc_pending_rewards: Decimal256,
    borrower_action: BorrowerActionResponse,
    loan_amount: Uint256,
}

impl Sdk {
    pub fn init() -> Self {
        let msg = basset_vault::basset_vault::InstantiateMsg {
            gov_addr: GOVERNANCE_CONTRACT.to_string(),
            community_addr: COMMUNITY_POOL_CONTRACT_ADDR.to_string(),
            nasset_t_ci: NASSET_TOKEN_CODE_ID,
            nasset_t_ch_ci: NASSET_TOKEN_CONFIG_HOLDER_CODE_ID,
            nasset_t_r_ci: NASSET_TOKEN_REWARDS_CODE_ID,
            psi_distr_ci: PSI_DISTRIBUTOR_CODE_ID,
            collateral_ts: COLLATERAL_TOKEN_SYMBOL.to_string(),
            basset_addr: BASSET_TOKEN_ADDR.to_string(),
            anchor_addr: ANCHOR_TOKEN.to_string(),
            a_market_addr: ANCHOR_MARKET_CONTRACT.to_string(),
            a_overseer_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
            a_custody_basset_addr: ANCHOR_CUSTODY_BASSET_CONTRACT.to_string(),
            a_basset_reward_addr: ANCHOR_BASSET_REWARD_CONTRACT.to_string(),
            anc_stable_swap_addr: ANC_STABLE_SWAP_CONTRACT.to_string(),
            psi_stable_swap_addr: PSI_STABLE_SWAP_CONTRACT.to_string(),
            ts_factory_addr: ASTROPORT_FACTORY_CONTRACT_ADDR.to_string(),
            aterra_addr: ATERRA_TOKEN.to_string(),
            psi_addr: PSI_TOKEN.to_string(),
            basset_vs_addr: BASSET_VAULT_STRATEGY_CONTRACT.to_string(),
            stable_denom: STABLE_DENOM.to_string(),
            claiming_rewards_delay: CLAIMING_REWARDS_DELAY,
            over_loan_balance_value: Decimal256::from_str(&OVER_LOAN_BALANCE_VALUE.to_string())
                .unwrap(),
            manual_ltv: Decimal256::from_str(&MANUAL_LTV.to_string()).unwrap(),
            fee_rate: Decimal256::from_str(&FEE_RATE.to_string()).unwrap(),
            tax_rate: Decimal256::from_str(&TAX_RATE.to_string()).unwrap(),
        };

        let mut deps = mock_dependencies(&[]);
        Self::instantiate_basset_vault(
            &mut deps,
            msg.clone(),
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
            anc_pending_rewards: Decimal256::zero(),
            borrower_action: BorrowerActionResponse::Nothing {},
            loan_amount: Uint256::zero(),
        }
    }

    pub fn instantiate_basset_vault<A: Storage, B: Api, C: Querier>(
        deps: &mut OwnedDeps<A, B, C>,
        init_msg: basset_vault::basset_vault::InstantiateMsg,
        nasset_token_config_holder_contract: &str,
        nasset_contract_addr: &str,
        nasset_token_rewards_contract: &str,
        psi_distributor_contract: &str,
    ) {
        let info = mock_info("addr9999", &[]);

        // ==========================================================
        // =================== Init BASSET_FARMER ===================
        // ========= Instantiate NASSET_TOKEN_CONFIG_HOLDER =========
        // ==========================================================
        {
            let res =
                crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg.clone())
                    .unwrap();
            assert_eq!(
                res.messages,
                vec![SubMsg {
                    msg: WasmMsg::Instantiate {
                        code_id: init_msg.nasset_t_ch_ci,
                        msg: to_binary(&NAssetTokenConfigHolderInstantiateMsg {
                            governance_contract_addr: init_msg.gov_addr.clone()
                        })
                        .unwrap(),
                        funds: vec![],
                        label: "".to_string(),
                        admin: Some(GOVERNANCE_CONTRACT.to_string()),
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAssetConfigHolder.id(),
                    reply_on: ReplyOn::Success,
                }]
            );
        }

        // ==========================================================
        // ============= Init NASSET_TOKEN_CONFIG_HOLDER ============
        // =============== Instantiate NASSET_TOKEN =================
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
                        code_id: init_msg.nasset_t_ci,
                        msg: to_binary(&NAssetTokenInstantiateMsg {
                            name: format!(
                                "Nexus b{} token share representation",
                                COLLATERAL_TOKEN_SYMBOL
                            ),
                            symbol: format!("n{}", COLLATERAL_TOKEN_SYMBOL),
                            decimals: 6,
                            initial_balances: vec![],
                            mint: Some(MinterResponse {
                                minter: MOCK_CONTRACT_ADDR.to_string(),
                                cap: None,
                            }),
                            marketing: None,
                            config_holder_contract: nasset_token_config_holder_contract.to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        label: "".to_string(),
                        admin: Some(GOVERNANCE_CONTRACT.to_string()),
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
        // =================== Init NASSET_TOKEN ====================
        // =============== Create nAsset <-> Psi pair ===============
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
                    msg: WasmMsg::Execute {
                        contract_addr: ASTROPORT_FACTORY_CONTRACT_ADDR.to_string(),
                        msg: to_binary(&AstroportFactoryExecuteMsg::CreatePair {
                            pair_type: PairType::Xyk {},
                            asset_infos: [
                                AssetInfo::Token {
                                    contract_addr: Addr::unchecked(nasset_contract_addr),
                                },
                                AssetInfo::Token {
                                    contract_addr: Addr::unchecked(PSI_TOKEN),
                                }
                            ],
                            init_params: None
                        })
                        .unwrap(),
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: SubmsgIds::InitNAssetPsiSwapPair.id(),
                    reply_on: ReplyOn::Always,
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

        // ===========================================================
        // =========== Init NASSET_PSI_SWAP_CONTRACT_ADDR ============
        // ============ Instantiate NASSET_TOKEN_REWARDS =============
        // ===========================================================
        {
            let reply_msg = Reply {
                id: SubmsgIds::InitNAssetPsiSwapPair.id(),
                result: cosmwasm_std::ContractResult::Ok(SubMsgExecutionResponse {
                    events: vec![Event::new("")
                        .add_attribute("pair_contract_addr", NASSET_PSI_SWAP_CONTRACT_ADDR)],
                    data: None,
                }),
            };

            let res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

            assert_eq!(
                res.messages,
                vec![SubMsg::reply_on_success(
                    CosmosMsg::Wasm(WasmMsg::Instantiate {
                        code_id: NASSET_TOKEN_REWARDS_CODE_ID,
                        msg: to_binary(&NAssetTokenRewardsInstantiateMsg {
                            psi_token_addr: PSI_TOKEN.to_string(),
                            nasset_token_addr: NASSET_TOKEN_ADDR.to_string(),
                            governance_contract_addr: GOVERNANCE_CONTRACT.to_string(),
                        })
                        .unwrap(),
                        funds: vec![],
                        label: "".to_string(),
                        admin: Some(GOVERNANCE_CONTRACT.to_string()),
                    }),
                    SubmsgIds::InitNAssetRewards.id(),
                )]
            );
            assert_eq!(
                res.attributes,
                vec![
                    attr("action", "nasset_psi_swap_pair_initialized"),
                    attr(
                        "nasset_psi_swap_contract_addr",
                        NASSET_PSI_SWAP_CONTRACT_ADDR
                    ),
                ]
            );
        }

        // ===========================================================================
        // ==== Pass NASSET_TOKEN_REWARDS_ADDR to psi distributor instantiate msg ====
        // ====================== Instantiate PSI_DISTRIBUTOR ========================
        // ===========================================================================
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
                            code_id: init_msg.psi_distr_ci,
                            msg: to_binary(&PsiDistributorInstantiateMsg {
                                psi_token_addr: PSI_TOKEN.to_string(),
                                nasset_token_rewards_contract_addr: nasset_token_rewards_contract
                                    .to_string(),
                                governance_contract_addr: init_msg.gov_addr.clone(),
                                community_pool_contract_addr: init_msg.community_addr.clone(),
                                basset_vault_strategy_contract_addr: init_msg
                                    .basset_vs_addr
                                    .clone(),
                                nasset_psi_swap_contract_addr: NASSET_PSI_SWAP_CONTRACT_ADDR
                                    .to_string(),
                                manual_ltv: init_msg.manual_ltv,
                                fee_rate: init_msg.fee_rate,
                                tax_rate: init_msg.tax_rate
                            })
                            .unwrap(),
                            funds: vec![],
                            label: "".to_string(),
                            admin: Some(GOVERNANCE_CONTRACT.to_string()),
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
        // ================== Init PSI_DISTRIBUTOR ==================
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
        self.loan_amount = value;
        self.set_wasm_query_respones();
    }

    pub fn set_tax(&mut self, tax_percent: Decimal, cap: u128) {
        self.deps.querier.with_tax(
            tax_percent,
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
                &BASSET_VAULT_STRATEGY_CONTRACT.to_string(),
                &to_binary(&0u64).unwrap(), //fake key, cause only one msg for this contract
                &to_binary(&self.borrower_action).unwrap(),
            ),
            (
                &ANCHOR_MARKET_CONTRACT.to_string(),
                &to_binary(&AnchorMarketQueryMsg::EpochState { block_height: None }).unwrap(),
                &to_binary(&AnchorMarketEpochStateResponse {
                    exchange_rate: self.aterra_exchange_rate,
                    aterra_supply: Uint256::from(1_000_000u64),
                })
                .unwrap(),
            ),
            (
                &ANCHOR_MARKET_CONTRACT.to_string(),
                &to_binary(&AnchorMarketQueryMsg::BorrowerInfo {
                    borrower: MOCK_CONTRACT_ADDR.to_string(),
                    block_height: None,
                })
                .unwrap(),
                &to_binary(&AnchorMarketBorrowerInfo {
                    borrower: MOCK_CONTRACT_ADDR.to_string(),
                    loan_amount: self.loan_amount,
                    pending_rewards: self.anc_pending_rewards,
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
                &AnchorBassetCustodyBorrowerInfo { balance },
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

    pub fn set_anc_pending_rewards(&mut self, value: Decimal256) {
        self.anc_pending_rewards = value;
        self.set_wasm_query_respones();
    }

    pub fn set_holding_pending_rewards(&mut self, value: Decimal256) {
        self.deps.querier.with_basset_holding_reward(value);
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
        let rebalance_msg = basset_vault::basset_vault::AnyoneMsg::Rebalance {};
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

    pub fn after_deposit_action(&mut self) -> StdResult<Response<Empty>> {
        let reply_msg = Reply {
            id: SubmsgIds::AfterDepositAction.id(),
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
            msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
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
            msg: to_binary(&Cw20HookMsg::Withdraw {}).unwrap(),
        };

        let info = mock_info(NASSET_TOKEN_ADDR, &vec![]);
        crate::contract::execute(
            self.deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Receive(cw20_deposit_msg),
        )
    }

    pub fn user_send_honest_work(&mut self) -> StdResult<Response<Empty>> {
        let honest_work_msg = basset_vault::basset_vault::AnyoneMsg::HonestWork {};
        let env = mock_env();
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
                yourself_msg: YourselfMsg::SwapAnc {},
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
                yourself_msg: YourselfMsg::DisributeRewards {},
            },
        )
    }
}
