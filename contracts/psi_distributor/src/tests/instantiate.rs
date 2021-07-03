use crate::state::{load_config, Config, RewardShare};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{testing::mock_dependencies, Addr};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = yield_optimizer::psi_distributor::InstantiateMsg {
        nasset_token_contract: "addr0000".to_string(),
        nasset_token_rewards_contract: "addr0001".to_string(),
        governance_contract: "addr0002".to_string(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    // it worked, let's query the state
    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(msg.nasset_token_contract, config.nasset_token_addr);
    assert_eq!(msg.governance_contract, config.governance_addr);
    let distribution = config.rewards_distribution.distribution();
    assert_eq!(
        *distribution,
        vec![
            RewardShare {
                recipient: Addr::unchecked(msg.nasset_token_rewards_contract),
                share: Decimal256::percent(70)
            },
            RewardShare {
                recipient: Addr::unchecked(msg.governance_contract),
                share: Decimal256::percent(30)
            }
        ]
    );
}
