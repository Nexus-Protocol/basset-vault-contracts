use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract: Addr,
    pub psi_distributor: Addr,
    pub anchor_token: Addr,
    pub anchor_overseer_contract: Addr,
    pub anchor_market_contract: Addr,
    pub anchor_custody_basset_contract: Addr,
    pub anc_stable_swap_contract: Addr,
    pub psi_stable_swap_contract: Addr,
    pub nasset_token: Addr,
    pub basset_token: Addr,
    pub aterra_token: Addr,
    pub psi_token: Addr,
    pub basset_farmer_config_contract: Addr,
    pub stable_denom: String,
    pub claiming_rewards_delay: u64,
    //UST value in balance should be more than loan
    //on what portion.
    //for example: 1.01 means 1% more than loan
    pub over_loan_balance_value: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct RepayingLoanState {
    pub iteration_index: u8,
    pub repayed_something: bool,
    pub to_repay_amount: Uint256,
    pub repaying_amount: Uint256,
    pub aim_buffer_size: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct ChildContractsInfo {
    pub nasset_token_code_id: u64,
    pub nasset_token_rewards_code_id: u64,
    pub psi_distributor_code_id: u64,
    pub collateral_token_symbol: String,
    pub nasset_token_holders_psi_rewards_share: u64,
    pub governance_contract_psi_rewards_share: u64,
}

static KEY_CONFIG: &[u8] = b"config";
static KEY_REPAYING_LOAN: &[u8] = b"repaying";
static KEY_AIM_BUFFER_SIZE: &[u8] = b"aim_buf_size";

static KEY_STABLE_BALANCE_BEFORE_SELL_ANC: &[u8] = b"balance_before_sell_anc";
static KEY_LAST_REWARDS_CLAIMING_HEIGHT: &[u8] = b"last_rewards_claiming_height";
//need that only for instantiating
static KEY_CHILD_CONTRACTS_INFO: &[u8] = b"child_contracts_code_id";
static KEY_NASSET_TOKEN_CONFIG_HOLDER: &[u8] = b"nasset_token_config_holder";

pub fn load_nasset_token_config_holder(storage: &dyn Storage) -> StdResult<Addr> {
    singleton_read(storage, KEY_NASSET_TOKEN_CONFIG_HOLDER).load()
}

pub fn store_nasset_token_config_holder(storage: &mut dyn Storage, addr: &Addr) -> StdResult<()> {
    singleton(storage, KEY_NASSET_TOKEN_CONFIG_HOLDER).save(addr)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn config_set_nasset_token(storage: &mut dyn Storage, nasset_token: Addr) -> StdResult<Config> {
    singleton(storage, KEY_CONFIG).update(|mut config: Config| -> StdResult<_> {
        config.nasset_token = nasset_token;
        Ok(config)
    })
}

pub fn config_set_psi_distributor(
    storage: &mut dyn Storage,
    psi_distributor: Addr,
) -> StdResult<Config> {
    singleton(storage, KEY_CONFIG).update(|mut config: Config| -> StdResult<_> {
        config.psi_distributor = psi_distributor;
        Ok(config)
    })
}

pub fn load_repaying_loan_state(storage: &dyn Storage) -> StdResult<RepayingLoanState> {
    singleton_read(storage, KEY_REPAYING_LOAN)
        .may_load()
        .map(|res| res.unwrap_or_default())
}

pub fn store_repaying_loan_state(
    storage: &mut dyn Storage,
    repaying_loan_state: &RepayingLoanState,
) -> StdResult<()> {
    singleton(storage, KEY_REPAYING_LOAN).save(repaying_loan_state)
}

pub fn update_loan_state_part_of_loan_repaid(
    storage: &mut dyn Storage,
) -> StdResult<RepayingLoanState> {
    singleton(storage, KEY_REPAYING_LOAN).update(
        |mut rep_loan: RepayingLoanState| -> StdResult<_> {
            rep_loan.to_repay_amount = rep_loan.to_repay_amount - rep_loan.repaying_amount;
            rep_loan.repayed_something = true;
            Ok(rep_loan)
        },
    )
}

pub fn load_aim_buffer_size(storage: &dyn Storage) -> StdResult<Uint256> {
    singleton_read(storage, KEY_AIM_BUFFER_SIZE).load()
}

pub fn store_aim_buffer_size(storage: &mut dyn Storage, aim_buf_size: &Uint256) -> StdResult<()> {
    singleton(storage, KEY_AIM_BUFFER_SIZE).save(aim_buf_size)
}

pub fn load_stable_balance_before_selling_anc(storage: &dyn Storage) -> StdResult<Uint128> {
    singleton_read(storage, KEY_STABLE_BALANCE_BEFORE_SELL_ANC).load()
}

pub fn store_stable_balance_before_selling_anc(
    storage: &mut dyn Storage,
    balance: &Uint128,
) -> StdResult<()> {
    singleton(storage, KEY_STABLE_BALANCE_BEFORE_SELL_ANC).save(balance)
}

pub fn load_child_contracts_info(storage: &dyn Storage) -> StdResult<ChildContractsInfo> {
    singleton_read(storage, KEY_CHILD_CONTRACTS_INFO).load()
}

pub fn store_child_contracts_info(
    storage: &mut dyn Storage,
    child_contracts_info: &ChildContractsInfo,
) -> StdResult<()> {
    singleton(storage, KEY_CHILD_CONTRACTS_INFO).save(child_contracts_info)
}

pub fn load_last_rewards_claiming_height(storage: &dyn Storage) -> StdResult<u64> {
    singleton_read(storage, KEY_LAST_REWARDS_CLAIMING_HEIGHT)
        .may_load()
        .map(|may_value| may_value.unwrap_or_default())
}

pub fn store_last_rewards_claiming_height(
    storage: &mut dyn Storage,
    height: &u64,
) -> StdResult<()> {
    singleton(storage, KEY_LAST_REWARDS_CLAIMING_HEIGHT).save(height)
}
