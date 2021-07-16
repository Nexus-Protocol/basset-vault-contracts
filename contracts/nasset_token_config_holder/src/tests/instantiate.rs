use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use basset_vault::nasset_token_config_holder::Config;

use crate::state::load_config;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let nasset_token_rewards_contract_addr = "addr0000".to_string();
    let msg = basset_vault::nasset_token_config_holder::InstantiateMsg {
        governance_contract_addr: "addr0001".to_string(),
    };

    let env = mock_env();
    let info = mock_info("addr0010", &[]);
    crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let config: Config = load_config(&deps.storage).unwrap();
    assert_eq!(msg.governance_contract_addr, config.governance_contract);
    assert_eq!("", config.nasset_token_rewards_contract);

    // set rewards_contract
    {
        let set_rewards_token_addr_msg = basset_vault::nasset_token_config_holder::ExecuteMsg::Anyone {
            anyone_msg:
                basset_vault::nasset_token_config_holder::AnyoneMsg::SetTokenRewardsContract {
                    nasset_token_rewards_contract_addr: nasset_token_rewards_contract_addr.clone(),
                },
        };
        crate::contract::execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            set_rewards_token_addr_msg.clone(),
        )
        .unwrap();
        let config: Config = load_config(&deps.storage).unwrap();
        assert_eq!(msg.governance_contract_addr, config.governance_contract);
        assert_eq!(
            nasset_token_rewards_contract_addr,
            config.nasset_token_rewards_contract
        );
    }

    // set rewards_contract second time
    {
        let set_rewards_token_addr_msg = basset_vault::nasset_token_config_holder::ExecuteMsg::Anyone {
            anyone_msg:
                basset_vault::nasset_token_config_holder::AnyoneMsg::SetTokenRewardsContract {
                    nasset_token_rewards_contract_addr: "no_way!".to_string(),
                },
        };
        let execute_res = crate::contract::execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            set_rewards_token_addr_msg.clone(),
        );
        assert!(execute_res.is_err());
        let config: Config = load_config(&deps.storage).unwrap();
        assert_eq!(msg.governance_contract_addr, config.governance_contract);
        assert_eq!(
            nasset_token_rewards_contract_addr,
            config.nasset_token_rewards_contract
        );
    }
}
