use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};

use crate::{querier::query_rewards_contract, ContractResult};
use cw20_base::allowances::{
    execute_burn_from as cw20_burn_from, execute_send_from as cw20_send_from,
    execute_transfer_from as cw20_transfer_from,
};
use cw20_base::contract::{
    execute_burn as cw20_burn, execute_mint as cw20_mint, execute_send as cw20_send,
    execute_transfer as cw20_transfer,
};
use basset_vault::nasset_token_rewards::{
    ExecuteMsg as NAssetRewardsExecuteMsg, TokenMsg as NassetRewardsTokenMsg,
};

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let sender = info.sender.to_string();
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response = cw20_transfer(deps, env, info, recipient.clone(), amount)?;

    Ok(Response {
        events: vec![],
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: rewards_contract.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                        address: sender,
                        amount,
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: rewards_contract.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                        address: recipient,
                        amount,
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
        ],
        attributes: res.attributes,
        data: None,
    })
}

pub fn burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response> {
    let sender = info.sender.to_string();
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response = cw20_burn(deps, env, info, amount)?;
    Ok(Response {
        events: vec![],
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: rewards_contract.to_string(),
            msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                    address: sender,
                    amount,
                },
            })
            .unwrap(),
            funds: vec![],
        }))],
        attributes: res.attributes,
        data: None,
    })
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response = cw20_mint(deps, env, info, recipient.clone(), amount)?;
    Ok(Response {
        events: vec![],
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: rewards_contract.to_string(),
            msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                    address: recipient,
                    amount,
                },
            })
            .unwrap(),
            funds: vec![],
        }))],
        attributes: res.attributes,
        data: None,
    })
}

pub fn send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> ContractResult<Response> {
    let sender = info.sender.to_string();
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response = cw20_send(deps, env, info, contract.clone(), amount, msg)?;
    Ok(Response {
        events: vec![],
        messages: vec![
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: rewards_contract.to_string(),
                    msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                        token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                            address: sender,
                            amount,
                        },
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: rewards_contract.to_string(),
                    msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                        token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                            address: contract,
                            amount,
                        },
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ],
            res.messages,
        ]
        .concat(),
        attributes: res.attributes,
        data: None,
    })
}

pub fn transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response =
        cw20_transfer_from(deps, env, info, owner.clone(), recipient.clone(), amount)?;
    Ok(Response {
        events: vec![],
        messages: vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: rewards_contract.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                        address: owner,
                        amount,
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: rewards_contract.to_string(),
                msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                    token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                        address: recipient,
                        amount,
                    },
                })
                .unwrap(),
                funds: vec![],
            })),
        ],
        attributes: res.attributes,
        data: None,
    })
}

pub fn burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> ContractResult<Response> {
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response = cw20_burn_from(deps, env, info, owner.clone(), amount)?;
    Ok(Response {
        events: vec![],
        messages: vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: rewards_contract.to_string(),
            msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                    address: owner,
                    amount,
                },
            })
            .unwrap(),
            funds: vec![],
        }))],
        attributes: res.attributes,
        data: None,
    })
}

pub fn send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> ContractResult<Response> {
    let rewards_contract = query_rewards_contract(deps.as_ref())?;

    let res: Response = cw20_send_from(
        deps,
        env,
        info,
        owner.clone(),
        contract.clone(),
        amount,
        msg,
    )?;

    Ok(Response {
        events: vec![],
        messages: vec![
            vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: rewards_contract.to_string(),
                    msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                        token_msg: NassetRewardsTokenMsg::DecreaseBalance {
                            address: owner,
                            amount,
                        },
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: rewards_contract.to_string(),
                    msg: to_binary(&NAssetRewardsExecuteMsg::Token {
                        token_msg: NassetRewardsTokenMsg::IncreaseBalance {
                            address: contract,
                            amount,
                        },
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            ],
            res.messages,
        ]
        .concat(),
        attributes: res.attributes,
        data: None,
    })
}
