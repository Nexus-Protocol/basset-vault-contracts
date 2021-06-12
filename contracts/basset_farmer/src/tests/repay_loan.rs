use crate::{
    contract::{execute, instantiate, reply},
    response::MsgInstantiateContractResponse,
};
use crate::{error::ContractError, state::load_farmer_info};

use crate::tests::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Reply, ReplyOn,
    Response, StdError, SubMsg, SubcallResponse, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use yield_optimizer::{
    basset_farmer::{Cw20HookMsg, ExecuteMsg, OverseerMsg},
    basset_farmer_config::BorrowerActionResponse,
    querier::BorrowerInfoResponse,
};

#[test]
fn repay_loan() {
    let cluna_contract_addr = "addr0001".to_string();
    let basset_token_addr = "addr0002".to_string();
    let custody_basset_contract = "addr0003".to_string();
    let overseer_addr = "addr0004".to_string();
    let governance_addr = "addr0005".to_string();
    let token_code_id = 10u64; //cw20 contract code
    let anchor_token = "addr0006".to_string();
    let anchor_market_contract = "addr0007".to_string();
    let anchor_ust_swap_contract = "addr0008".to_string();
    let ust_psi_swap_contract = "addr0009".to_string();
    let aterra_token = "addr0010".to_string();
    let psi_part_in_rewards = Decimal::from_ratio(1u64, 100u64);
    let psi_token = "addr0011".to_string();
    let basset_farmer_config_contract = "addr0012".to_string();
    let stable_denom = "addr0013".to_string();
    let mut deps = mock_dependencies(&[]);

    //basset_farmer and custody_bluna have zero 'cluna' coins
    deps.querier.with_token_balances(&[(
        &cluna_contract_addr,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
    )]);

    // -= INITIALIZATION =-
    {
        let init_msg = yield_optimizer::basset_farmer::InstantiateMsg {
            token_code_id,
            collateral_token_symbol: "Luna".to_string(),
            basset_token_addr: basset_token_addr.clone(),
            custody_basset_contract: custody_basset_contract.clone(),
            overseer_addr: overseer_addr.clone(),
            governance_addr: governance_addr.to_string(),
            anchor_token,
            anchor_market_contract: anchor_market_contract.clone(),
            anchor_ust_swap_contract,
            ust_psi_swap_contract,
            aterra_token,
            psi_part_in_rewards,
            psi_token,
            basset_farmer_config_contract: basset_farmer_config_contract.clone(),
            stable_denom,
        };

        let info = mock_info("addr0000", &[]);
        let _res = crate::contract::instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        let mut cw20_instantiate_response = MsgInstantiateContractResponse::new();
        cw20_instantiate_response.set_contract_address(cluna_contract_addr.clone());

        // store cLuna token address
        let reply_msg = Reply {
            id: 1,
            result: ContractResult::Ok(SubcallResponse {
                events: vec![],
                data: Some(cw20_instantiate_response.write_to_bytes().unwrap().into()),
            }),
        };

        let _res = crate::contract::reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();
    }

    let locked_basset_amount = Uint128::from(10_000u64);
    let basset_farmer_loan_amount = Uint256::from(10_000u64);
    deps.querier.with_token_balances(&[(
        &custody_basset_contract,
        &[(&MOCK_CONTRACT_ADDR.to_string(), &locked_basset_amount)],
    )]);
    deps.querier.with_loan(&[(
        &anchor_market_contract,
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &BorrowerInfoResponse {
                borrower: MOCK_CONTRACT_ADDR.to_string(),
                interest_index: Decimal256::one(),
                reward_index: Decimal256::zero(),
                loan_amount: basset_farmer_loan_amount,
                pending_rewards: Decimal256::zero(),
            },
        )],
    )]);
    deps.querier.with_wasm_query_response(&[(
        &basset_farmer_config_contract,
        &to_binary(&BorrowerActionResponse::Repay {
            amount: Uint256::from(10_000u64),
            advised_buffer_size: Uint256::from(5_000u64),
        })
        .unwrap(),
    )]);
    // -= REBALANCE =-
    {
        let rebalance_msg = yield_optimizer::basset_farmer::AnyoneMsg::Rebalance;
        let info = mock_info("addr8888", &vec![]);
        let _res = crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Anyone {
                anyone_msg: rebalance_msg,
            },
        )
        .unwrap();
    }
}
