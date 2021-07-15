use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, AllBalanceResponse, BalanceResponse, BankQuery, Binary, Coin, Deps,
    QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::TokenInfoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn query_balance(
    querier: &QuerierWrapper,
    account_addr: &Addr,
    denom: String,
) -> StdResult<Uint128> {
    // load price form the oracle
    let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr.to_string(),
        denom,
    }))?;
    Ok(balance.amount.amount)
}

pub fn query_all_balances(querier: &QuerierWrapper, account_addr: Addr) -> StdResult<Vec<Coin>> {
    // load price form the oracle
    let all_balances: AllBalanceResponse =
        querier.query(&QueryRequest::Bank(BankQuery::AllBalances {
            address: account_addr.to_string(),
        }))?;
    Ok(all_balances.amount)
}

pub fn query_token_balance(
    deps: Deps,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    Ok(deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr.to_string(),
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                (deps.api.addr_canonicalize(account_addr.as_str())?).as_slice(),
            )),
        }))
        .unwrap_or_else(|_| Uint128::zero()))
}

pub fn query_supply(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    let token_info: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.to_string(),
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    Ok(token_info.total_supply)
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfoResponse {
    pub borrower: String,
    pub loan_amount: Uint256,
    pub pending_rewards: Decimal256,
    // we do not need those fields, removing it will save some space in
    // compiled wasm file
    //
    // pub interest_index: Decimal256,
    // pub reward_index: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfo {
    pub loan_amount: Uint256,
    pub pending_rewards: Decimal256,
    // we do not need those fields, removing it will save some space in
    // compiled wasm file
    //
    // pub interest_index: Decimal256,
    // pub reward_index: Decimal256,
}

pub fn query_borrower_info(
    deps: Deps,
    anchor_market_contract: &Addr,
    borrower: &Addr,
) -> StdResult<BorrowerInfoResponse> {
    let borrower_info: BorrowerInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: anchor_market_contract.to_string(),
        key: Binary::from(concat(
            &to_length_prefixed(b"liability").to_vec(),
            (deps.api.addr_canonicalize(borrower.as_str())?).as_slice(),
        )),
    }))?;

    Ok(BorrowerInfoResponse {
        borrower: borrower.to_string(),
        loan_amount: borrower_info.loan_amount,
        pending_rewards: borrower_info.pending_rewards,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerResponse {
    pub balance: Uint256,
    // we do not need those fields, removing it will save some space in
    // compiled wasm file
    //
    // pub borrower: String,
    // pub spendable: Uint256,
}

pub fn get_basset_in_custody(
    deps: Deps,
    custody_basset_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint256> {
    let borrower_info: BorrowerResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: custody_basset_addr.to_string(),
            key: Binary::from(concat(
                &to_length_prefixed(b"borrower").to_vec(),
                (deps.api.addr_canonicalize(account_addr.as_str())?).as_slice(),
            )),
        }))?;

    Ok(borrower_info.balance)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMarketMsg {
    ClaimRewards {
        to: Option<String>,
    },
    DepositStable,
    BorrowStable {
        borrow_amount: Uint256,
        to: Option<String>,
    },
    RepayStable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMarketCw20Msg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMarketQueryMsg {
    EpochState { block_height: Option<u64> },
    // using Raw query to ask for state
    // State { block_height: Option<u64> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AnchorMarketEpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

pub fn query_aterra_state(
    deps: Deps,
    anchor_market_contract: &Addr,
) -> StdResult<AnchorMarketEpochStateResponse> {
    let epoch_state: AnchorMarketEpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: anchor_market_contract.to_string(),
            msg: to_binary(&AnchorMarketQueryMsg::EpochState { block_height: None })?,
        }))?;

    Ok(epoch_state)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AnchorMarketStateResponse {
    pub total_liabilities: Decimal256,
    pub total_reserves: Decimal256,
    // we do not need those fields, removing it will save some space in
    // compiled wasm file
    //
    // pub last_interest_updated: u64,
    // pub last_reward_updated: u64,
    // pub global_interest_index: Decimal256,
    // pub global_reward_index: Decimal256,
    // pub anc_emission_rate: Decimal256,
}

pub fn query_market_state(
    deps: Deps,
    anchor_market_contract: &Addr,
) -> StdResult<AnchorMarketStateResponse> {
    let market_state: AnchorMarketStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: anchor_market_contract.to_string(),
            key: Binary::from(to_length_prefixed(b"state").to_vec()),
        }))?;

    Ok(market_state)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AnchorMarketConfigResponse {
    // we do not need those fields, removing it will save some space in
    // compiled wasm file
    //
    // pub contract_addr: CanonicalAddr,
    // pub owner_addr: CanonicalAddr,
    // pub aterra_contract: CanonicalAddr,
    // pub interest_model: CanonicalAddr,
    // pub distribution_model: CanonicalAddr,
    // pub overseer_contract: CanonicalAddr,
    // pub collector_contract: CanonicalAddr,
    // pub distributor_contract: CanonicalAddr,
    // pub stable_denom: String,
    // pub reserve_factor: Decimal256,
    pub max_borrow_factor: Decimal256,
}

pub fn query_market_config(
    deps: Deps,
    anchor_market_contract: &Addr,
) -> StdResult<AnchorMarketConfigResponse> {
    let market_config: AnchorMarketConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: anchor_market_contract.to_string(),
            key: Binary::from(to_length_prefixed(b"config").to_vec()),
        }))?;

    Ok(market_config)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnchorOverseerMsg {
    LockCollateral { collaterals: Vec<(String, Uint256)> },
    UnlockCollateral { collaterals: Vec<(String, Uint256)> },
}
