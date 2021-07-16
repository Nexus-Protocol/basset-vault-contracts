use cosmwasm_std::{to_binary, Api, CosmosMsg, Querier, Storage, SubMsg, Uint128, WasmMsg};
use cosmwasm_std::{DepsMut, OwnedDeps};

use cw20::{Cw20ReceiveMsg, MinterResponse, TokenInfoResponse};
use cw20_base::contract::{query_minter, query_token_info};
use cw20_base::msg::ExecuteMsg;
use basset_vault::nasset_token::InstantiateMsg as TokenInstantiateMsg;
use basset_vault::nasset_token_rewards::{
    ExecuteMsg as NAssetRewardsExecuteMsg, TokenMsg as NassetRewardsTokenMsg,
};

use super::mock_dependencies;
use super::{MOCK_CONFIG_HOLDER_CONTRACT_ADDR, MOCK_OWNER_ADDR, MOCK_REWARDS_CONTRACT_ADDR};
use crate::{
    contract::{execute, instantiate},
    state::load_config_holder_contract,
};
use cosmwasm_std::testing::{mock_env, mock_info};

const CANONICAL_LENGTH: usize = 20;

// this will set up the init for other tests
fn do_init_with_minter<A: Storage, B: Api, C: Querier>(
    deps: &mut OwnedDeps<A, B, C>,
    minter: String,
    cap: Option<Uint128>,
) -> TokenInfoResponse {
    _do_init(deps, Some(MinterResponse { minter, cap }))
}

// this will set up the init for other tests
fn _do_init<A: Storage, B: Api, C: Querier>(
    deps: &mut OwnedDeps<A, B, C>,
    mint: Option<MinterResponse>,
) -> TokenInfoResponse {
    let instantiate_msg = TokenInstantiateMsg {
        name: "nluna".to_string(),
        symbol: "NLUNA".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: mint.clone(),
        config_holder_contract: MOCK_CONFIG_HOLDER_CONTRACT_ADDR.to_string(),
    };

    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let meta = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "nluna".to_string(),
            symbol: "NLUNA".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );
    assert_eq!(query_minter(deps.as_ref()).unwrap(), mint);
    meta
}

pub fn do_mint(deps: DepsMut, addr: String, amount: Uint128) {
    let msg = ExecuteMsg::Mint {
        recipient: addr,
        amount,
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let res = execute(deps, mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let instantiate_msg = TokenInstantiateMsg {
        name: "nluna".to_string(),
        symbol: "NLUNA".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: MOCK_OWNER_ADDR.to_string(),
            cap: None,
        }),
        config_holder_contract: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
    };
    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "nluna".to_string(),
            symbol: "NLUNA".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );

    assert_eq!(
        query_minter(deps.as_ref()).unwrap(),
        Some(MinterResponse {
            minter: MOCK_OWNER_ADDR.to_string(),
            cap: None
        })
    );

    assert_eq!(
        load_config_holder_contract(&deps.storage).unwrap(),
        MOCK_REWARDS_CONTRACT_ADDR.to_string()
    );
}

#[test]
fn transfer() {
    let mut deps = mock_dependencies(&[]);
    let addr1 = "addr0001".to_string();
    let addr2 = "addr0002".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                        address: addr1.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                        address: addr2.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );
}

#[test]
fn transfer_from() {
    let mut deps = mock_dependencies(&[]);
    let addr1 = "addr0001".to_string();
    let addr2 = "addr0002".to_string();
    let addr3 = "addr0003".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr3.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info(&addr3, &[]);
    let msg = ExecuteMsg::TransferFrom {
        owner: addr1.clone(),
        recipient: addr2.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                        address: addr1.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                        address: addr2.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );
}

#[test]
fn mint() {
    let mut deps = mock_dependencies(&[]);
    let addr = "addr0000".to_string();

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);

    let info = mock_info(MOCK_OWNER_ADDR, &[]);
    let msg = ExecuteMsg::Mint {
        recipient: addr.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
            msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                    address: addr.clone(),
                    amount: Uint128::new(1u128),
                },
            })
            .unwrap(),
            funds: vec![],
        }),)]
    );
}

#[test]
fn burn() {
    let mut deps = mock_dependencies(&[]);
    let addr = "addr0000".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr.clone(), amount1);

    let info = mock_info(&addr, &[]);
    let msg = ExecuteMsg::Burn {
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
            msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                    address: addr.clone(),
                    amount: Uint128::new(1u128),
                },
            })
            .unwrap(),
            funds: vec![],
        }),)]
    );
}

#[test]
fn burn_from() {
    let mut deps = mock_dependencies(&[]);
    let addr = "addr0000".to_string();
    let addr1 = "addr0001".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr.clone(), amount1);

    let info = mock_info(&addr, &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr1.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::BurnFrom {
        owner: addr.clone(),
        amount: Uint128::new(1u128),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
            msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                    address: addr.clone(),
                    amount: Uint128::new(1u128),
                },
            })
            .unwrap(),
            funds: vec![],
        }),)]
    );
}

#[test]
fn send() {
    let mut deps = mock_dependencies(&[]);
    let addr1 = "addr0001".to_string();
    let dummny_contract_addr = "dummy".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let dummy_msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1u128),
    };

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::Send {
        contract: dummny_contract_addr.clone(),
        amount: Uint128::new(1u128),
        msg: to_binary(&dummy_msg).unwrap(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0..2].to_vec(),
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                        address: addr1.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                        address: dummny_contract_addr.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );
    assert_eq!(
        res.messages[2],
        SubMsg::new(
            Cw20ReceiveMsg {
                sender: addr1.clone(),
                amount: Uint128::new(1),
                msg: to_binary(&dummy_msg).unwrap()
            }
            .into_cosmos_msg(dummny_contract_addr)
            .unwrap()
        )
    );
}

#[test]
fn send_from() {
    let mut deps = mock_dependencies(&[]);
    let addr1 = "addr0001".to_string();
    let addr2 = "addr0002".to_string();
    let dummny_contract_addr = "dummy".to_string();
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(&mut deps, MOCK_OWNER_ADDR.to_string(), None);
    do_mint(deps.as_mut(), addr1.clone(), amount1);

    let info = mock_info(&addr1, &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr2.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let dummy_msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1u128),
    };

    let info = mock_info(&addr2, &[]);
    let msg = ExecuteMsg::SendFrom {
        owner: addr1.clone(),
        contract: dummny_contract_addr.clone(),
        amount: Uint128::new(1u128),
        msg: to_binary(&dummy_msg).unwrap(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0..2].to_vec(),
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                        address: addr1.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_REWARDS_CONTRACT_ADDR.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                        address: dummny_contract_addr.clone(),
                        amount: Uint128::new(1u128),
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
        ]
    );

    assert_eq!(
        res.messages[2],
        SubMsg::new(
            Cw20ReceiveMsg {
                sender: addr2.clone(),
                amount: Uint128::new(1),
                msg: to_binary(&dummy_msg).unwrap(),
            }
            .into_cosmos_msg(dummny_contract_addr)
            .unwrap()
        )
    );
}
