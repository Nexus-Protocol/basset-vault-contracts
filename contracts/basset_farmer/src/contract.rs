use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};

use crate::{commands, queries};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{
    state::{Config, CONFIG},
    ContractResult,
};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use yield_optimizer::basset_farmer::{
    AnyoneMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, OverseerMsg, QueryMsg,
    YourselfMsg,
};

pub const SUBMSG_ID_INIT_CASSET: u64 = 1;
pub const SUBMSG_ID_REDEEM_STABLE: u64 = 2;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        casset_token: Addr::unchecked(""),
        basset_token: deps.api.addr_validate(&msg.basset_token_addr)?,
        overseer_contract: deps.api.addr_validate(&msg.overseer_addr)?,
        custody_basset_contract: deps.api.addr_validate(&msg.custody_basset_contract)?,
        governance_contract: deps.api.addr_validate(&msg.governance_addr)?,

        //TODO: get from InstantiateMsg
        anchor_token: Addr::unchecked(""),
        anchor_market_contract: Addr::unchecked(""),
        anchor_ust_swap_contract: Addr::unchecked(""),
        ust_psi_swap_contract: Addr::unchecked(""),
        aterra_token: Addr::unchecked(""),
        psi_part_in_rewards: Uint128::from(0u64),
        psi_token: Addr::unchecked(""),
        basset_farmer_config_contract: Addr::unchecked(""),
        stable_denom: "".to_string(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Instantiate {
                admin: None,
                code_id: msg.token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "nexus basset token share representation".to_string(),
                    symbol: format!("c{}", msg.collateral_token_symbol),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })?,
                send: vec![],
                label: "".to_string(),
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_CASSET,
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![],
        data: None,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    //TODO: add check for msg.id:
    //if SUBMSG_ID_INIT_CASSET then ...
    let data = msg.result.unwrap().data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let casset_token = res.get_contract_address();

    let api = deps.api;
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.casset_token = api.addr_validate(casset_token)?;
        Ok(config)
    })?;

    Ok(Response {
        messages: vec![],
        submessages: vec![],
        attributes: vec![attr("casset_token_addr", casset_token)],
        data: None,
    })
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Yourself { yourself_msg } => match yourself_msg {
            YourselfMsg::AfterBorrow {
                borrowed_amount,
                buffer_size,
            } => todo!(),
            YourselfMsg::AfterAterraRedeem { repay_amount } => todo!(),
        },
        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::Rebalance {} => commands::rebalance(deps, env, info),
            AnyoneMsg::Sweep {} => commands::sweep(deps, env, info),
            AnyoneMsg::SwapAnc {} => commands::swap_anc(deps, env, info),
            AnyoneMsg::BuyPsiTokens {} => commands::buy_psi_tokens(deps, env, info),
            AnyoneMsg::DisributeRewards {} => commands::distribute_rewards(deps, env, info),
            AnyoneMsg::ClaimRewards {} => commands::claim_rewards(deps, env, info),
        },
        ExecuteMsg::OverseerMsg { overseer_msg } => match overseer_msg {
            OverseerMsg::Deposit { farmer, amount } => {
                commands::deposit_basset(deps, env, info, farmer, amount)
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
