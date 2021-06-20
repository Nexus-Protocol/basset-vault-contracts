use std::default;

use commands::repay_logic_on_reply;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};

use crate::{
    commands, queries,
    state::{
        config_set_casset_token, load_aim_buffer_size, load_config, load_repaying_loan_state,
        store_config, store_state, update_loan_state_part_of_loan_repaid, RepayingLoanState, State,
    },
};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{state::Config, ContractResult};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use protobuf::Message;
use yield_optimizer::casset_staking::{AnyoneMsg, ExecuteMsg, MigrateMsg, QueryMsg};

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
        anchor_token: deps.api.addr_validate(&msg.anchor_token)?,
        anchor_market_contract: deps.api.addr_validate(&msg.anchor_market_contract)?,
        anchor_ust_swap_contract: deps.api.addr_validate(&msg.anchor_ust_swap_contract)?,
        ust_psi_swap_contract: deps.api.addr_validate(&msg.ust_psi_swap_contract)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_token)?,
        psi_part_in_rewards: msg.psi_part_in_rewards,
        psi_token: deps.api.addr_validate(&msg.psi_token)?,
        basset_farmer_config_contract: deps
            .api
            .addr_validate(&msg.basset_farmer_config_contract)?,
        stable_denom: msg.stable_denom,
    };
    store_config(deps.storage, &config)?;

    let state = State {
        global_reward_index: Decimal256::zero(),
        last_reward_amount: Decimal256::zero(),
    };
    store_state(deps.storage, &state)?;

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
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    Ok(Response::default())
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
        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::UpdateIndex => commands::update_global_index(deps, env, info),

            AnyoneMsg::ClaimRewards => {
                todo!()
            }

            AnyoneMsg::Unstake { amount, to } => {
                commands::unstake_casset(deps, env, info.sender, amount, to)
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&queries::query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
