use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};

use crate::{
    commands, queries,
    state::{load_config, load_farmer_info, store_farmer_info, FarmerInfo},
};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use yield_optimizer::{
    basset_farmer::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    querier::{get_basset_in_custody, query_supply, query_token_balance},
};

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => commands::receive_cw20_deposit(deps, env, info, cw20_msg),
        Ok(Cw20HookMsg::Withdraw {}) => commands::receive_cw20_withdraw(deps, env, info, cw20_msg),
        Err(err) => Err(ContractError::Std(err)),
    }
}

pub fn receive_cw20_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let basset_addr = info.sender;
    // only basset contract can execute this message
    let config: Config = CONFIG.load(deps.storage)?;
    if basset_addr != config.basset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    receive_basset(deps, farmer_addr, cw20_msg.amount.into())
}

pub fn receive_basset(deps: DepsMut, farmer: Addr, amount: Uint256) -> ContractResult<Response> {
    let mut farmer_info: FarmerInfo = load_farmer_info(deps.storage, &farmer)?;

    farmer_info.spendable_basset += amount;

    store_farmer_info(deps.storage, &farmer, &farmer_info)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![
            attr("action", "deposit_basset_step_1"),
            attr("farmer", farmer.as_str()),
            attr("amount", amount.to_string()),
        ],
        data: None,
    })
}

pub fn receive_cw20_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ContractResult<Response> {
    let contract_addr = info.sender;
    // only basset contract can execute this message
    let config: Config = CONFIG.load(deps.storage)?;
    if contract_addr != config.basset_token {
        return Err(ContractError::Unauthorized {});
    }

    //we trust cw20 contract
    let farmer_addr: Addr = Addr::unchecked(cw20_msg.sender);

    withdrawn_basset(deps, farmer_addr, cw20_msg.amount.into())
}

pub fn withdrawn_basset(deps: DepsMut, farmer: Addr, amount: Uint256) -> ContractResult<Response> {
    //TODO

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![],
        data: None,
    })
}

/// Executor: overseer
pub fn deposit_basset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    farmer: String,
    deposit_amount: Uint256,
) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    if info.sender != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }

    let farmer_addr: Addr = deps.api.addr_validate(&farmer)?;
    let mut farmer_info: FarmerInfo = load_farmer_info(deps.storage, &farmer_addr)?;
    if deposit_amount > farmer_info.spendable_basset {
        return Err(StdError::generic_err(format!(
            "Deposit amount cannot excceed the user's spendable amount: {}",
            farmer_info.spendable_basset
        ))
        .into());
    }

    // total cAsset supply
    let casset_supply: Uint256 = query_supply(&deps.querier, config.casset_token.clone())?.into();

    // basset balance in custody contract
    let basset_in_custody = get_basset_in_custody(
        deps.as_ref(),
        config.custody_basset_contract,
        env.contract.address.clone(),
    )?;

    // basset balance in cw20 contract
    let bluna_in_contract_address =
        query_token_balance(deps.as_ref(), config.basset_token, env.contract.address)?;

    // cAsset tokens to mint:
    // user_share = (deposited_basset / total_basset)
    // cAsset_to_mint = cAsset_supply * user_share / (1 - user_share)
    let basset_balance: Uint256 = basset_in_custody + bluna_in_contract_address.into();
    if basset_balance == Uint256::zero() {
        //impossible because if 'farmer' have 'spendable_basset' then he deposit some bAsset
        return Err(ContractError::Impossible(
            "basset balance is zero".to_string(),
        ));
    }
    let farmer_basset_share: Decimal256 =
        Decimal256::from_ratio(deposit_amount.0, basset_balance.0);

    let casset_to_mint = if farmer_basset_share == Decimal256::one() {
        deposit_amount
    } else {
        // 'casset_supply' can't be zero here, cause we already mint some for first farmer
        casset_supply * farmer_basset_share / (Decimal256::one() - farmer_basset_share)
    };

    farmer_info.spendable_basset = farmer_info.spendable_basset - deposit_amount;
    farmer_info.balance_casset += casset_to_mint;
    store_farmer_info(deps.storage, &farmer_addr, &farmer_info)?;

    Ok(Response {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.casset_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: farmer.clone(),
                //TODO: what is the reason to use Uint256 if we convert it to Uint128 at the end?
                amount: casset_to_mint.into(),
            })?,
            send: vec![],
        })],
        submessages: vec![],
        attributes: vec![
            attr("action", "deposit_basset_step_2"),
            attr("farmer", farmer),
            attr("amount", deposit_amount),
        ],
        data: None,
    })
}
