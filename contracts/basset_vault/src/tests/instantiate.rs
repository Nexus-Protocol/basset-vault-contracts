use crate::{
    state::{
        load_child_contracts_info, load_config, load_psi_distributor_init_info, ChildContractsInfo,
        Config, PsiDistributorInitInfo,
    },
    tests::sdk::{
        ANCHOR_CUSTODY_BASSET_CONTRACT, ANCHOR_MARKET_CONTRACT, ANCHOR_OVERSEER_CONTRACT,
        ANCHOR_TOKEN, ANC_STABLE_SWAP_CONTRACT, ASTROPORT_FACTORY_CONTRACT_ADDR, ATERRA_TOKEN,
        BASSET_TOKEN_ADDR, BASSET_VAULT_STRATEGY_CONTRACT, CLAIMING_REWARDS_DELAY,
        COLLATERAL_TOKEN_SYMBOL, COMMUNITY_POOL_CONTRACT_ADDR, FEE_RATE, GOVERNANCE_CONTRACT,
        MANUAL_LTV, NASSET_PSI_SWAP_CONTRACT_ADDR, NASSET_TOKEN_ADDR, NASSET_TOKEN_CODE_ID,
        NASSET_TOKEN_REWARDS_CODE_ID, OVER_LOAN_BALANCE_VALUE, PSI_DISTRIBUTOR_CODE_ID,
        PSI_DISTRIBUTOR_CONTRACT, PSI_STABLE_SWAP_CONTRACT, PSI_TOKEN, STABLE_DENOM, TAX_RATE,
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
            governance_contract: Addr::unchecked(GOVERNANCE_CONTRACT),
            anchor_token: Addr::unchecked(ANCHOR_TOKEN),
            anchor_overseer_contract: Addr::unchecked(ANCHOR_OVERSEER_CONTRACT),
            anchor_market_contract: Addr::unchecked(ANCHOR_MARKET_CONTRACT),
            anchor_custody_basset_contract: Addr::unchecked(ANCHOR_CUSTODY_BASSET_CONTRACT),
            anc_stable_swap_contract: Addr::unchecked(ANC_STABLE_SWAP_CONTRACT),
            psi_stable_swap_contract: Addr::unchecked(PSI_STABLE_SWAP_CONTRACT),
            basset_token: Addr::unchecked(BASSET_TOKEN_ADDR),
            aterra_token: Addr::unchecked(ATERRA_TOKEN),
            psi_token: Addr::unchecked(PSI_TOKEN),
            basset_vault_strategy_contract: Addr::unchecked(BASSET_VAULT_STRATEGY_CONTRACT),
            stable_denom: STABLE_DENOM.to_string(),
            claiming_rewards_delay: CLAIMING_REWARDS_DELAY,
            over_loan_balance_value: Decimal256::from_str(OVER_LOAN_BALANCE_VALUE).unwrap(),
            nasset_token: Addr::unchecked(NASSET_TOKEN_ADDR),
            psi_distributor: Addr::unchecked(PSI_DISTRIBUTOR_CONTRACT),
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
            community_pool_contract_addr: COMMUNITY_POOL_CONTRACT_ADDR.to_string(),
            manual_ltv: Decimal256::from_str(MANUAL_LTV).unwrap(),
            fee_rate: Decimal256::from_str(FEE_RATE).unwrap(),
            tax_rate: Decimal256::from_str(TAX_RATE).unwrap(),
        }
    );

    let psi_distributor_init_info =
        load_psi_distributor_init_info(sdk.deps.as_ref().storage).unwrap();
    assert_eq!(
        psi_distributor_init_info,
        PsiDistributorInitInfo {
            terraswap_factory_contract_addr: ASTROPORT_FACTORY_CONTRACT_ADDR.to_string(),
            nasset_psi_swap_contract_addr: Some(NASSET_PSI_SWAP_CONTRACT_ADDR.to_string()),
        }
    )
}
