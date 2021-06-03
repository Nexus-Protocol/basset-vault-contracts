use crate::contract::{execute, instantiate, reply};
use crate::error::ContractError;

use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Reply, ReplyOn,
    Response, StdError, SubMsg, SubcallResponse, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

#[test]
fn provide_liquidity() {
    // let mut deps = mock_dependencies(&[Coin {
    //     denom: "uusd".to_string(),
    //     amount: Uint128(200u128),
    // }]);

    // deps.querier.with_token_balances(&[(
    //     &"liquidity0000".to_string(),
    //     &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(0))],
    // )]);

    // let msg = InstantiateMsg {
    //     asset_infos: [
    //         AssetInfo::NativeToken {
    //             denom: "uusd".to_string(),
    //         },
    //         AssetInfo::Token {
    //             contract_addr: Addr::unchecked("asset0000"),
    //         },
    //     ],
    //     token_code_id: 10u64,
    // };

    // let env = mock_env();
    // let info = mock_info("addr0000", &[]);
    // // we can just call .unwrap() to assert this was a success
    // let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // // store liquidity token
    // let reply_msg = Reply {
    //     id: 1,
    //     result: ContractResult::Ok(SubcallResponse {
    //         events: vec![],
    //         data: Some(
    //             vec![
    //                 10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
    //             ]
    //             .into(),
    //         ),
    //     }),
    // };

    // let _res = reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

    // // successfully provide liquidity for the exist pool
    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //     ],
    //     slippage_tolerance: None,
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0000",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(100u128),
    //     }],
    // );
    // let res = execute(deps.as_mut(), env, info, msg).unwrap();
    // let transfer_from_msg = res.messages.get(0).expect("no message");
    // let mint_msg = res.messages.get(1).expect("no message");
    // assert_eq!(
    //     transfer_from_msg,
    //     &CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "asset0000".to_string(),
    //         msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
    //             owner: "addr0000".to_string(),
    //             recipient: MOCK_CONTRACT_ADDR.to_string(),
    //             amount: Uint128::from(100u128),
    //         })
    //         .unwrap(),
    //         send: vec![],
    //     })
    // );
    // assert_eq!(
    //     mint_msg,
    //     &CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "liquidity0000".to_string(),
    //         msg: to_binary(&Cw20ExecuteMsg::Mint {
    //             recipient: "addr0000".to_string(),
    //             amount: Uint128::from(100u128),
    //         })
    //         .unwrap(),
    //         send: vec![],
    //     })
    // );

    // // provide more liquidity 1:2, which is not proportional to 1:1,
    // // then it must accept 1:1 and treat left amount as donation
    // deps.querier.with_balance(&[(
    //     &MOCK_CONTRACT_ADDR.to_string(),
    //     vec![Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128(200 + 200 /* user deposit must be pre-applied */),
    //     }],
    // )]);

    // deps.querier.with_token_balances(&[
    //     (
    //         &"liquidity0000".to_string(),
    //         &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(100))],
    //     ),
    //     (
    //         &"asset0000".to_string(),
    //         &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(200))],
    //     ),
    // ]);

    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(200u128),
    //         },
    //     ],
    //     slippage_tolerance: None,
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0000",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(200u128),
    //     }],
    // );

    // // only accept 100, then 50 share will be generated with 100 * (100 / 200)
    // let res: Response = execute(deps.as_mut(), env, info, msg).unwrap();
    // let transfer_from_msg = res.messages.get(0).expect("no message");
    // let mint_msg = res.messages.get(1).expect("no message");
    // assert_eq!(
    //     transfer_from_msg,
    //     &CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "asset0000".to_string(),
    //         msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
    //             owner: "addr0000".to_string(),
    //             recipient: MOCK_CONTRACT_ADDR.to_string(),
    //             amount: Uint128::from(100u128),
    //         })
    //         .unwrap(),
    //         send: vec![],
    //     })
    // );
    // assert_eq!(
    //     mint_msg,
    //     &CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "liquidity0000".to_string(),
    //         msg: to_binary(&Cw20ExecuteMsg::Mint {
    //             recipient: "addr0000".to_string(),
    //             amount: Uint128::from(50u128),
    //         })
    //         .unwrap(),
    //         send: vec![],
    //     })
    // );

    // // check wrong argument
    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(50u128),
    //         },
    //     ],
    //     slippage_tolerance: None,
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0000",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(100u128),
    //     }],
    // );
    // let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    // match res {
    //     ContractError::Std(StdError::GenericErr { msg, .. }) => assert_eq!(
    //         msg,
    //         "Native token balance mismatch between the argument and the transferred".to_string()
    //     ),
    //     _ => panic!("Must return generic error"),
    // }

    // // initialize token balance to 1:1
    // deps.querier.with_balance(&[(
    //     &MOCK_CONTRACT_ADDR.to_string(),
    //     vec![Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128(100 + 100 /* user deposit must be pre-applied */),
    //     }],
    // )]);

    // deps.querier.with_token_balances(&[
    //     (
    //         &"liquidity0000".to_string(),
    //         &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(100))],
    //     ),
    //     (
    //         &"asset0000".to_string(),
    //         &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128(100))],
    //     ),
    // ]);

    // // failed because the price is under slippage_tolerance
    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(98u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //     ],
    //     slippage_tolerance: Some(Decimal::percent(1)),
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0001",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(100u128),
    //     }],
    // );
    // let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    // match res {
    //     ContractError::MaxSlippageAssertion {} => {}
    //     _ => panic!("DO NOT ENTER HERE"),
    // }

    // // initialize token balance to 1:1
    // deps.querier.with_balance(&[(
    //     &MOCK_CONTRACT_ADDR.to_string(),
    //     vec![Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128(100 + 98 /* user deposit must be pre-applied */),
    //     }],
    // )]);

    // // failed because the price is under slippage_tolerance
    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(98u128),
    //         },
    //     ],
    //     slippage_tolerance: Some(Decimal::percent(1)),
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0001",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(98u128),
    //     }],
    // );
    // let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    // match res {
    //     ContractError::MaxSlippageAssertion {} => {}
    //     _ => panic!("DO NOT ENTER HERE"),
    // }

    // // initialize token balance to 1:1
    // deps.querier.with_balance(&[(
    //     &MOCK_CONTRACT_ADDR.to_string(),
    //     vec![Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128(100 + 100 /* user deposit must be pre-applied */),
    //     }],
    // )]);

    // // successfully provides
    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(99u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //     ],
    //     slippage_tolerance: Some(Decimal::percent(1)),
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0001",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(100u128),
    //     }],
    // );
    // let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // // initialize token balance to 1:1
    // deps.querier.with_balance(&[(
    //     &MOCK_CONTRACT_ADDR.to_string(),
    //     vec![Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128(100 + 99 /* user deposit must be pre-applied */),
    //     }],
    // )]);

    // // successfully provides
    // let msg = ExecuteMsg::ProvideLiquidity {
    //     assets: [
    //         Asset {
    //             info: AssetInfo::Token {
    //                 contract_addr: Addr::unchecked("asset0000".to_string()),
    //             },
    //             amount: Uint128::from(100u128),
    //         },
    //         Asset {
    //             info: AssetInfo::NativeToken {
    //                 denom: "uusd".to_string(),
    //             },
    //             amount: Uint128::from(99u128),
    //         },
    //     ],
    //     slippage_tolerance: Some(Decimal::percent(1)),
    // };

    // let env = mock_env();
    // let info = mock_info(
    //     "addr0001",
    //     &[Coin {
    //         denom: "uusd".to_string(),
    //         amount: Uint128::from(99u128),
    //     }],
    // );
    // let _res = execute(deps.as_mut(), env, info, msg).unwrap();
}
