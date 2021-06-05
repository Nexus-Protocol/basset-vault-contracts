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
use yield_optimizer::basset_farmer::{Cw20HookMsg, ExecuteMsg, OverseerMsg};

#[test]
fn deposit_basset() {
    let cluna_contract_addr = "addr0001".to_string();
    let basset_token_addr = "addr0002".to_string();
    let custody_basset_contract = "addr0003".to_string();
    let overseer_addr = "addr0004".to_string();
    let governance_addr = "addr0005".to_string();
    let token_code_id = 10u64; //cw20 contract code
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

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint128 = 1000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_farmer =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_1_address.clone(),
                amount: deposit_1_amount,
                msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
            };

            let info = mock_info(&basset_token_addr, &vec![]);
            let _res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            let farmer_info =
                load_farmer_info(&deps.storage, &Addr::unchecked(user_1_address.clone())).unwrap();
            assert_eq!(
                Uint256::from(deposit_1_amount),
                farmer_info.spendable_basset
            );
        }

        deps.querier.with_token_balances(&[
            (
                &cluna_contract_addr,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
            ),
            (
                &basset_token_addr,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
        ]);

        // -= OVERSEER SEND 'Deposit' message to basset_farmer =-
        {
            let deposit_msg = OverseerMsg::Deposit {
                farmer: user_1_address.clone(),
                amount: deposit_1_amount.into(),
            };
            let info = mock_info(&overseer_addr, &vec![]);
            let _res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::OverseerMsg {
                    overseer_msg: deposit_msg,
                },
            )
            .unwrap();

            let farmer_info =
                load_farmer_info(&deps.storage, &Addr::unchecked(user_1_address)).unwrap();
            assert_eq!(Uint256::zero(), farmer_info.spendable_basset);
            assert_eq!(Uint256::from(deposit_1_amount), farmer_info.balance_casset);
        }
    }

    //second farmer come
    let user_2_address = "addr6666".to_string();
    let deposit_2_amount: Uint128 = 2000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_farmer =-
        {
            let cw20_deposit_msg = Cw20ReceiveMsg {
                sender: user_2_address.clone(),
                amount: deposit_2_amount,
                msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
            };

            let info = mock_info(&basset_token_addr, &vec![]);
            let _res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::Receive(cw20_deposit_msg),
            )
            .unwrap();

            let farmer_info =
                load_farmer_info(&deps.storage, &Addr::unchecked(user_2_address.clone())).unwrap();
            assert_eq!(
                Uint256::from(deposit_2_amount),
                farmer_info.spendable_basset
            );
        }

        deps.querier.with_token_balances(&[
            (
                &cluna_contract_addr,
                &[(&MOCK_CONTRACT_ADDR.to_string(), &deposit_1_amount)],
            ),
            (
                &basset_token_addr,
                &[(
                    &MOCK_CONTRACT_ADDR.to_string(),
                    &(deposit_2_amount + deposit_1_amount),
                )],
            ),
        ]);

        // -= OVERSEER SEND 'Deposit' message to basset_farmer =-
        {
            let deposit_msg = OverseerMsg::Deposit {
                farmer: user_2_address.clone(),
                amount: deposit_2_amount.into(),
            };
            let info = mock_info(&overseer_addr, &vec![]);
            let _res = crate::contract::execute(
                deps.as_mut(),
                mock_env(),
                info,
                ExecuteMsg::OverseerMsg {
                    overseer_msg: deposit_msg,
                },
            )
            .unwrap();

            let farmer_info =
                load_farmer_info(&deps.storage, &Addr::unchecked(user_2_address)).unwrap();
            assert_eq!(Uint256::zero(), farmer_info.spendable_basset);

            let farmer_share = Decimal256::from_ratio(
                Uint256::from(deposit_2_amount).0,
                Uint256::from(deposit_1_amount + deposit_2_amount).0,
            );
            let casset_supply = deposit_1_amount;
            let expected_farmer_casset_amount =
                Uint256::from(casset_supply) * farmer_share / (Decimal256::one() - farmer_share);
            assert_eq!(
                Uint256::from(expected_farmer_casset_amount),
                farmer_info.balance_casset
            );
        }
    }
}
