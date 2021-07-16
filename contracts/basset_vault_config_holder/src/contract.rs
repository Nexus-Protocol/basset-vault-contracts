use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::state::{load_config, save_config};
use crate::ContractResult;
use std::str::FromStr;
use basset_vault::basset_vault_config_holder::{
    Config, ConfigResponse, ExecuteMsg, GovernanceMsg, InstantiateMsg, QueryMsg,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        governance_contract: deps.api.addr_validate(&msg.governance_contract_addr)?,
        anchor_token: deps.api.addr_validate(&msg.anchor_token_addr)?,
        anchor_overseer_contract: deps.api.addr_validate(&msg.anchor_overseer_contract_addr)?,
        anchor_market_contract: deps.api.addr_validate(&msg.anchor_market_contract_addr)?,
        anchor_custody_basset_contract: deps
            .api
            .addr_validate(&msg.anchor_custody_basset_contract_addr)?,
        anc_stable_swap_contract: deps.api.addr_validate(&msg.anc_stable_swap_contract_addr)?,
        psi_stable_swap_contract: deps.api.addr_validate(&msg.psi_stable_swap_contract_addr)?,
        basset_token: deps.api.addr_validate(&msg.basset_token_addr)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_token_addr)?,
        psi_token: deps.api.addr_validate(&msg.psi_token_addr)?,
        basset_vault_strategy_contract: deps
            .api
            .addr_validate(&msg.basset_vault_strategy_contract_addr)?,
        stable_denom: msg.stable_denom.clone(),
        claiming_rewards_delay: msg.claiming_rewards_delay,
        over_loan_balance_value: Decimal256::from_str(&msg.over_loan_balance_value)?,
    };

    save_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Governance { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(ContractError::Unauthorized);
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    governance_contract_addr,
                    anchor_overseer_contract_addr,
                    anchor_market_contract_addr,
                    anchor_custody_basset_contract_addr,
                    anc_stable_swap_contract_addr,
                    psi_stable_swap_contract_addr,
                    basset_vault_strategy_contract_addr,
                    claiming_rewards_delay,
                    over_loan_balance_value,
                } => update_config(
                    deps,
                    config,
                    governance_contract_addr,
                    anchor_overseer_contract_addr,
                    anchor_market_contract_addr,
                    anchor_custody_basset_contract_addr,
                    anc_stable_swap_contract_addr,
                    psi_stable_swap_contract_addr,
                    basset_vault_strategy_contract_addr,
                    claiming_rewards_delay,
                    over_loan_balance_value,
                ),
            }
        }
    }
}

fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    governance_contract_addr: Option<String>,
    anchor_overseer_contract_addr: Option<String>,
    anchor_market_contract_addr: Option<String>,
    anchor_custody_basset_contract_addr: Option<String>,
    anc_stable_swap_contract_addr: Option<String>,
    psi_stable_swap_contract_addr: Option<String>,
    basset_vault_strategy_contract_addr: Option<String>,
    claiming_rewards_delay: Option<u64>,
    over_loan_balance_value: Option<String>,
) -> ContractResult<Response> {
    if let Some(ref governance_addr) = governance_contract_addr {
        current_config.governance_contract = deps.api.addr_validate(governance_addr)?;
    }

    if let Some(ref anchor_overseer_contract_addr) = anchor_overseer_contract_addr {
        current_config.anchor_overseer_contract =
            deps.api.addr_validate(anchor_overseer_contract_addr)?;
    }

    if let Some(ref anchor_market_contract_addr) = anchor_market_contract_addr {
        current_config.anchor_market_contract =
            deps.api.addr_validate(anchor_market_contract_addr)?;
    }

    if let Some(ref anchor_custody_basset_contract_addr) = anchor_custody_basset_contract_addr {
        current_config.anchor_custody_basset_contract = deps
            .api
            .addr_validate(anchor_custody_basset_contract_addr)?;
    }

    if let Some(ref anc_stable_swap_contract_addr) = anc_stable_swap_contract_addr {
        current_config.anc_stable_swap_contract =
            deps.api.addr_validate(anc_stable_swap_contract_addr)?;
    }

    if let Some(ref psi_stable_swap_contract_addr) = psi_stable_swap_contract_addr {
        current_config.psi_stable_swap_contract =
            deps.api.addr_validate(psi_stable_swap_contract_addr)?;
    }

    if let Some(ref basset_vault_strategy_contract_addr) = basset_vault_strategy_contract_addr {
        current_config.basset_vault_strategy_contract = deps
            .api
            .addr_validate(basset_vault_strategy_contract_addr)?;
    }

    if let Some(ref claiming_rewards_delay) = claiming_rewards_delay {
        current_config.claiming_rewards_delay = claiming_rewards_delay.clone();
    }

    if let Some(ref over_loan_balance_value) = over_loan_balance_value {
        current_config.over_loan_balance_value = Decimal256::from_str(over_loan_balance_value)?;
    }

    save_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        governance_contract_addr: config.governance_contract.to_string(),
        anchor_token_addr: config.anchor_token.to_string(),
        anchor_overseer_contract_addr: config.anchor_overseer_contract.to_string(),
        anchor_market_contract_addr: config.anchor_market_contract.to_string(),
        anchor_custody_basset_contract_addr: config.anchor_custody_basset_contract.to_string(),
        anc_stable_swap_contract_addr: config.anc_stable_swap_contract.to_string(),
        psi_stable_swap_contract_addr: config.psi_stable_swap_contract.to_string(),
        basset_token_addr: config.basset_token.to_string(),
        aterra_token_addr: config.aterra_token.to_string(),
        psi_token_addr: config.psi_token.to_string(),
        basset_vault_strategy_contract_addr: config.basset_vault_strategy_contract.to_string(),
        stable_denom: config.stable_denom.clone(),
        claiming_rewards_delay: config.claiming_rewards_delay,
        over_loan_balance_value: config.over_loan_balance_value,
    })
}
