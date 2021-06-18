use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract: Addr,
    pub overseer_contract: Addr,
    pub anchor_token: Addr,
    pub anchor_market_contract: Addr,
    pub custody_basset_contract: Addr,
    //remove UST from name
    pub anchor_ust_swap_contract: Addr,
    //remove UST from name
    pub ust_psi_swap_contract: Addr,
    pub casset_token: Addr,
    pub basset_token: Addr,
    pub aterra_token: Addr,
    //what part of UST from selling ANC spend to buy PSI
    pub psi_part_in_rewards: Decimal,
    pub psi_token: Addr,
    pub basset_farmer_config_contract: Addr,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // pub last_reward_updated: u64,
    pub global_reward_index: Decimal256,
    pub last_reward_amount: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct RepayingLoanState {
    pub iteration_index: u8,
    pub to_repay_amount: Uint256,
    pub repaying_amount: Uint256,
    pub aim_buffer_size: Uint256,
}

const CONFIG: Item<Config> = Item::new("config");
const STATE: Item<State> = Item::new("state");
const REPAYING_LOAN: Item<RepayingLoanState> = Item::new("repaying");
const FARMERS: Map<&Addr, FarmerInfo> = Map::new("farmers");
const AIM_BUFFER_SIZE: Item<Uint256> = Item::new("aim_buf_size");
const STABLE_BALANCE_BEFORE_SELL_ANC: Item<Uint128> = Item::new("balance_before_sell_anc");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct FarmerInfo {
    //TODO: probably I don't need that field. We know balance from cAsset contract address
    //but I leave it here for some time
    pub balance_casset: Uint256,
    pub spendable_basset: Uint256,
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn config_set_casset_token(storage: &mut dyn Storage, casset_token: Addr) -> StdResult<Config> {
    CONFIG.update(storage, |mut config| -> StdResult<_> {
        config.casset_token = casset_token;
        Ok(config)
    })
}

pub fn load_farmer_info(storage: &dyn Storage, farmer_addr: &Addr) -> StdResult<FarmerInfo> {
    FARMERS
        .may_load(storage, farmer_addr)
        .map(|res| res.unwrap_or_default())
}

pub fn store_farmer_info(
    storage: &mut dyn Storage,
    farmer_addr: &Addr,
    farmer_info: &FarmerInfo,
) -> StdResult<()> {
    FARMERS.save(storage, farmer_addr, farmer_info)
}

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn load_repaying_loan_state(storage: &dyn Storage) -> StdResult<RepayingLoanState> {
    REPAYING_LOAN
        .may_load(storage)
        .map(|res| res.unwrap_or_default())
}

pub fn store_repaying_loan_state(
    storage: &mut dyn Storage,
    repaying_loan_state: &RepayingLoanState,
) -> StdResult<()> {
    REPAYING_LOAN.save(storage, repaying_loan_state)
}

pub fn update_loan_state_part_of_loan_repaid(
    storage: &mut dyn Storage,
) -> StdResult<RepayingLoanState> {
    REPAYING_LOAN.update(storage, |mut rep_loan| -> StdResult<_> {
        rep_loan.to_repay_amount = rep_loan.to_repay_amount - rep_loan.repaying_amount;
        Ok(rep_loan)
    })
}

pub fn load_aim_buffer_size(storage: &dyn Storage) -> StdResult<Uint256> {
    AIM_BUFFER_SIZE.load(storage)
}

pub fn store_aim_buffer_size(storage: &mut dyn Storage, aim_buf_size: &Uint256) -> StdResult<()> {
    AIM_BUFFER_SIZE.save(storage, aim_buf_size)
}

pub fn load_stable_balance_before_selling_anc(storage: &dyn Storage) -> StdResult<Uint128> {
    STABLE_BALANCE_BEFORE_SELL_ANC.load(storage)
}

pub fn store_stable_balance_before_selling_anc(
    storage: &mut dyn Storage,
    balance: &Uint128,
) -> StdResult<()> {
    STABLE_BALANCE_BEFORE_SELL_ANC.save(storage, balance)
}
