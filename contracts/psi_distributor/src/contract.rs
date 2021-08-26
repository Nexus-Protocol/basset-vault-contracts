use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, Config},
    ContractResult,
};
use basset_vault::psi_distributor::{
    AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let one = Decimal256::one();
    if msg.manual_ltv >= one || msg.fee_rate >= one || msg.tax_rate >= one {
        return Err(
            StdError::generic_err("none of decimal numbers can be bigger or equal to one").into(),
        );
    }

    let config = Config {
        psi_token: deps.api.addr_validate(&msg.psi_token_addr)?,
        governance_contract: deps.api.addr_validate(&msg.governance_contract_addr)?,
        nasset_token_rewards_contract: deps
            .api
            .addr_validate(&msg.nasset_token_rewards_contract_addr)?,
        community_pool_contract: deps.api.addr_validate(&msg.community_pool_contract_addr)?,
        basset_vault_strategy_contract: deps
            .api
            .addr_validate(&msg.basset_vault_strategy_contract_addr)?,
        manual_ltv: msg.manual_ltv,
        fee_rate: msg.fee_rate,
        tax_rate: msg.tax_rate,
    };
    store_config(deps.storage, &config)?;

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
        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::DistributeRewards {} => commands::distribute_rewards(deps, env),
        },

        ExecuteMsg::GovernanceMsg { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    governance_contract_addr,
                    nasset_token_rewards_contract_addr,
                    community_pool_contract_addr,
                    basset_vault_strategy_contract_addr,
                    manual_ltv,
                    fee_rate,
                    tax_rate,
                } => commands::update_config(
                    deps,
                    config,
                    governance_contract_addr,
                    nasset_token_rewards_contract_addr,
                    community_pool_contract_addr,
                    basset_vault_strategy_contract_addr,
                    manual_ltv,
                    fee_rate,
                    tax_rate,
                ),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
    }
}
