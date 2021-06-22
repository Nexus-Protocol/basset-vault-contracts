use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, WasmQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub governance_contract_addr: String,
    pub oracle_addr: String,
    pub basset_token_addr: String,
    pub stable_denom: String,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub basset_max_ltv: Decimal256,
    pub buffer_part: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    GovernanceMsg { governance_msg: GovernanceMsg },
}

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
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config,
    BorrowerAction {
        borrowed_amount: Uint256,
        locked_basset_amount: Uint256,
    },
}

//TODO: update, cause Config struct changed
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub governance_contract_addr: Addr,
    pub oracle_addr: Addr,
    pub basset_token_addr: Addr,
    pub stable_denom: String,
    pub borrow_ltv_max: Decimal256,
    pub borrow_ltv_min: Decimal256,
    pub borrow_ltv_aim: Decimal256,
    pub basset_max_ltv: Decimal256,
    pub buffer_part: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BorrowerActionResponse {
    Nothing,
    Borrow {
        amount: Uint256,
        advised_buffer_size: Uint256,
    },
    Repay {
        amount: Uint256,
        advised_buffer_size: Uint256,
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
        BorrowerActionResponse::Nothing
    }
}

pub fn query_borrower_action(
    deps: Deps,
    basset_farmer_config_contract: &Addr,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
) -> StdResult<BorrowerActionResponse> {
    let borrower_action: BorrowerActionResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: basset_farmer_config_contract.to_string(),
            msg: to_binary(&QueryMsg::BorrowerAction {
                borrowed_amount,
                locked_basset_amount,
            })?,
        }))?;

    Ok(borrower_action)
}
