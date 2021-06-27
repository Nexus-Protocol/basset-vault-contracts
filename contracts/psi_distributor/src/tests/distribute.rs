use crate::tests::mock_dependencies;
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_binary, Uint128,
};
use cosmwasm_std::{CosmosMsg, WasmMsg};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use yield_optimizer::psi_distributor::{AnyoneMsg, ExecuteMsg};

#[test]
fn distribute_rewards() {
    let mut deps = mock_dependencies(&[]);
    let nasset_token_contract = "addr0001".to_string();
    let nasset_staker_contract = "addr0002".to_string();
    let governance_addr = "addr0003".to_string();

    let msg = yield_optimizer::psi_distributor::InstantiateMsg {
        nasset_token_contract: nasset_token_contract.clone(),
        nasset_staker_contract: nasset_staker_contract.clone(),
        governance_contract: governance_addr.clone(),
    };

    // -= INITIALIZATION =-
    {
        let env = mock_env();
        let info = mock_info("addr0010", &[]);
        crate::contract::instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
    }

    //contract have some "rewards"
    deps.querier.with_token_balances(&[(
        &nasset_token_contract,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(1000))],
    )]);

    // -= DISTRIBUTE =-
    {
        let distribute_msg = ExecuteMsg::Anyone {
            anyone_msg: AnyoneMsg::DistributeRewards,
        };
        let info = mock_info("addr0000", &[]);
        let res =
            crate::contract::execute(deps.as_mut(), mock_env(), info, distribute_msg).unwrap();

        assert_eq!(
            res.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: nasset_token_contract.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: nasset_staker_contract.clone(),
                        amount: Uint128::from(700u64),
                    })
                    .unwrap(),
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: nasset_token_contract.clone(),
                    send: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: governance_addr.clone(),
                        amount: Uint128::from(300u64),
                    })
                    .unwrap(),
                }),
            ]
        );
    }
}
