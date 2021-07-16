use crate::{
    state::{load_child_contracts_info, load_config, ChildContractsInfo, Config},
    tests::sdk::{
        BASSET_FARMER_CONFIG_HOLDER_CONTRACT, COLLATERAL_TOKEN_SYMBOL,
        GOVERNANCE_STAKER_REWARDS_SHARE, NASSET_TOKEN_ADDR, NASSET_TOKEN_CODE_ID,
        NASSET_TOKEN_HOLDERS_REWARDS_SHARE, NASSET_TOKEN_REWARDS_CODE_ID, PSI_DISTRIBUTOR_CODE_ID,
        PSI_DISTRIBUTOR_CONTRACT,
    },
};

use super::sdk::Sdk;
use cosmwasm_std::Addr;

#[test]
fn proper_initialization() {
    let sdk = Sdk::init();

    let config = load_config(sdk.deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            nasset_token: Addr::unchecked(NASSET_TOKEN_ADDR),
            config_holder: Addr::unchecked(BASSET_FARMER_CONFIG_HOLDER_CONTRACT),
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
            nasset_token_holders_psi_rewards_share: NASSET_TOKEN_HOLDERS_REWARDS_SHARE,
            governance_contract_psi_rewards_share: GOVERNANCE_STAKER_REWARDS_SHARE,
        }
    );
}
