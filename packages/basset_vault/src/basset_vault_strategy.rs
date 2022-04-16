use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, WasmQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance_contract_addr: String,
    pub oracle_contract_addr: String,
    pub anchor_market_addr: String,
    pub anchor_interest_model_addr: String,
    pub anchor_overseer_addr: String,
    pub anc_ust_swap_addr: String,
    pub anchor_token_addr: String,
    pub basset_token_addr: String,
    pub stable_denom: String,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub basset_max_ltv: Decimal256,
    pub buffer_part: Decimal256,
    pub price_timeframe: u64,
    pub staking_apr: Decimal256,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Governance { governance_msg: GovernanceMsg },
    Anyone { anyone_msg: AnyoneMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnyoneMsg {
    AcceptGovernance {},
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceMsg {
    UpdateConfig {
        oracle_addr: Option<String>,
        basset_token_addr: Option<String>,
        stable_denom: Option<String>,
        borrow_ltv_max: Option<Decimal256>,
        borrow_ltv_min: Option<Decimal256>,
        borrow_ltv_aim: Option<Decimal256>,
        basset_max_ltv: Option<Decimal256>,
        buffer_part: Option<Decimal256>,
        price_timeframe: Option<u64>,
        anchor_market_addr: Option<String>,
        anchor_interest_model_addr: Option<String>,
        anchor_overseer_addr: Option<String>,
        anc_ust_swap_addr: Option<String>,
        anchor_token_addr: Option<String>,
        staking_apr: Option<Decimal256>,
    },
    UpdateGovernanceContract {
        gov_addr: String,
        //how long to wait for 'AcceptGovernance' transaction
        seconds_to_wait_for_accept_gov_tx: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BorrowerAction {
        basset_in_contract_address: Uint256,
        borrowed_amount: Uint256,
        locked_basset_amount: Uint256,
    },
    AnchorApr {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract: String,
    pub oracle_contract: String,
    pub anchor_market_contract: String,
    pub anchor_interest_model_contract: String,
    pub anchor_overseer_contract: String,
    pub anc_ust_swap_contract: String,
    pub anchor_token_contract: String,
    pub basset_token: String,
    pub stable_denom: String,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub basset_max_ltv: Decimal256,
    pub buffer_part: Decimal256,
    pub price_timeframe: u64,
    pub staking_apr: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BorrowerActionResponse {
    Nothing {},
    Borrow {
        amount: Uint256,
        advised_buffer_size: Uint256,
    },
    Repay {
        amount: Uint256,
        advised_buffer_size: Uint256,
    },
    Deposit {
        deposit_amount: Uint256,
        /// We need to rebalance again after deposit
        action_after: Box<BorrowerActionResponse>,
    },
    RepayAllAndWithdraw {
        withdraw_amount: Uint256,
    },
}

impl BorrowerActionResponse {
    pub fn repay(amount: Uint256, advised_buffer_size: Uint256) -> Self {
        BorrowerActionResponse::Repay {
            amount,
            advised_buffer_size,
        }
    }

    pub fn borrow(amount: Uint256, advised_buffer_size: Uint256) -> Self {
        BorrowerActionResponse::Borrow {
            amount,
            advised_buffer_size,
        }
    }

    pub fn nothing() -> Self {
        BorrowerActionResponse::Nothing {}
    }

    pub fn deposit(deposit_amount: Uint256, action_after: BorrowerActionResponse) -> Self {
        BorrowerActionResponse::Deposit {
            deposit_amount,
            action_after: Box::new(action_after),
        }
    }

    pub fn repay_all_and_withdraw(withdraw_amount: Uint256) -> Self {
        BorrowerActionResponse::RepayAllAndWithdraw {
            withdraw_amount
        }
    }
}

pub fn query_borrower_action(
    deps: Deps,
    basset_vault_strategy_contract: &Addr,
    basset_in_contract_address: Uint256,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
) -> StdResult<BorrowerActionResponse> {
    let borrower_action: BorrowerActionResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: basset_vault_strategy_contract.to_string(),
            msg: to_binary(&QueryMsg::BorrowerAction {
                basset_in_contract_address,
                borrowed_amount,
                locked_basset_amount,
            })?,
        }))?;

    Ok(borrower_action)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub anchor_market_addr: String,
    pub anchor_interest_model_addr: String,
    pub anchor_overseer_addr: String,
    pub anc_ust_swap_addr: String,
    pub anchor_token_addr: String,
    pub staking_apr: Decimal256,
}
