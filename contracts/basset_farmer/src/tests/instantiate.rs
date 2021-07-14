use crate::{
    state::{load_child_contracts_info, load_config, ChildContractsInfo, Config},
    tests::sdk::{
        ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT,
        ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, ATERRA_TOKEN, BASSET_FARMER_CONFIG_CONTRACT,
        BASSET_TOKEN_ADDR, CLAIMING_REWARDS_DELAY, COLLATERAL_TOKEN_SYMBOL, GOVERNANCE_CONTRACT,
        GOVERNANCE_STAKER_REWARDS_SHARE, NASSET_TOKEN_ADDR, NASSET_TOKEN_CODE_ID,
        NASSET_TOKEN_HOLDERS_REWARDS_SHARE, NASSET_TOKEN_REWARDS_CODE_ID, OVER_LOAN_BALANCE_VALUE,
        PSI_DISTRIBUTOR_CODE_ID, PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, PSI_TOKEN,
        STABLE_DENOM,
    },
};

use super::sdk::Sdk;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::Addr;
use std::str::FromStr;

#[test]
fn proper_initialization() {
    let sdk = Sdk::init();

    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            nasset_token: Addr::unchecked(NASSET_TOKEN_ADDR),
            basset_token: Addr::unchecked(BASSET_TOKEN_ADDR),
            anchor_custody_basset_contract: Addr::unchecked(ANCHOR_CUSTODY_BASSET_CONTRACT),
            governance_contract: Addr::unchecked(GOVERNANCE_CONTRACT),
            anchor_token: Addr::unchecked(ANCHOR_TOKEN),
            anchor_overseer_contract: Addr::unchecked(ANCHOR_OVERSEER_CONTRACT),
            anchor_market_contract: Addr::unchecked(ANCHOR_MARKET_CONTRACT),
            anc_stable_swap_contract: Addr::unchecked(ANC_STABLE_SWAP_CONTRACT),
            psi_stable_swap_contract: Addr::unchecked(PSI_STABLE_SWAP_CONTRACT),
            aterra_token: Addr::unchecked(ATERRA_TOKEN),
            psi_token: Addr::unchecked(PSI_TOKEN),
            basset_farmer_strategy_contract: Addr::unchecked(BASSET_FARMER_CONFIG_CONTRACT),
            stable_denom: STABLE_DENOM.to_string(),
            claiming_rewards_delay: CLAIMING_REWARDS_DELAY,
            psi_distributor: Addr::unchecked(PSI_DISTRIBUTOR_CONTRACT),
            over_loan_balance_value: Decimal256::from_str(&OVER_LOAN_BALANCE_VALUE).unwrap(),
        }
    );

    let child_contracts_info = load_child_contracts_info(sdk.deps.as_ref().storage).unwrap();
    assert_eq!(
        child_contracts_info,
        ChildContractsInfo {
            nasset_token_code_id: NASSET_TOKEN_CODE_ID,
            nasset_token_rewards_code_id: NASSET_TOKEN_REWARDS_CODE_ID,
            psi_distributor_code_id: PSI_DISTRIBUTOR_CODE_ID,
            collateral_token_symbol: COLLATERAL_TOKEN_SYMBOL.to_string(),
            nasset_token_holders_psi_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
            governance_contract_psi_rewards_share: GOVERNANCE_STAKER_REWARDS_SHARE,
        }
    );
}
