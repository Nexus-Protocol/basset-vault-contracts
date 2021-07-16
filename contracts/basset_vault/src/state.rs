use cosmwasm_storage::{singleton, singleton_read, to_length_prefixed};
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, Binary, Deps, QueryRequest, StdResult, Storage, Uint128, WasmQuery};
use basset_vault::basset_vault_config_holder::Config as ExternalConfig;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub config_holder: Addr,
    pub nasset_token: Addr,
    pub psi_distributor: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct RepayingLoanState {
    pub iteration_index: u8,
    pub repayed_something: bool,
    pub to_repay_amount: Uint256,
    pub repaying_amount: Uint256,
    pub aim_buffer_size: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
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

pub fn query_external_config(deps: Deps) -> StdResult<ExternalConfig> {
    let config_holder_contract = load_config(deps.storage)?;
    query_external_config_light(deps, &config_holder_contract)
}

pub fn query_external_config_light(deps: Deps, config: &Config) -> StdResult<ExternalConfig> {
    let config: ExternalConfig = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: config.config_holder.to_string(),
        key: Binary::from(to_length_prefixed(b"config")),
    }))?;

    Ok(config)
}
