use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, load_legacy_config, save_config, Config},
    ContractResult,
};
use basset_vault::psi_distributor::{
    AnyoneMsg, ExecuteMsg, GovernanceMsg, InstantiateMsg, MigrateMsg, QueryMsg,
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
        nasset_psi_swap_contract_addr: deps
            .api
            .addr_validate(&msg.nasset_psi_swap_contract_addr)?,
        manual_ltv: msg.manual_ltv,
        fee_rate: msg.fee_rate,
        tax_rate: msg.tax_rate,
    };
    save_config(deps.storage, &config)?;

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
            AnyoneMsg::AcceptGovernance {} => commands::accept_governance(deps, env, info),
        },

        ExecuteMsg::Governance { governance_msg } => {
            let config = load_config(deps.storage)?;
            if info.sender != config.governance_contract {
                return Err(ContractError::Unauthorized {});
            }

            match governance_msg {
                GovernanceMsg::UpdateConfig {
                    nasset_token_rewards_contract_addr,
                    community_pool_contract_addr,
                    basset_vault_strategy_contract_addr,
                    nasset_psi_swap_contract_addr,
                    manual_ltv,
                    fee_rate,
                    tax_rate,
                } => commands::update_config(
                    deps,
                    config,
                    nasset_token_rewards_contract_addr,
                    community_pool_contract_addr,
                    basset_vault_strategy_contract_addr,
                    nasset_psi_swap_contract_addr,
                    manual_ltv,
                    fee_rate,
                    tax_rate,
                ),

                GovernanceMsg::UpdateGovernanceContract {
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
                } => commands::update_governance_addr(
                    deps,
                    env,
                    gov_addr,
                    seconds_to_wait_for_accept_gov_tx,
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

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let legacy_config = load_legacy_config(deps.storage)?;
    let config = Config {
        psi_token: legacy_config.psi_token,
        governance_contract: legacy_config.governance_contract,
        nasset_token_rewards_contract: legacy_config.nasset_token_rewards_contract,
        community_pool_contract: legacy_config.community_pool_contract,
        basset_vault_strategy_contract: legacy_config.basset_vault_strategy_contract,
        nasset_psi_swap_contract_addr: deps
            .api
            .addr_validate(&msg.nasset_psi_swap_contract_addr)?,
        manual_ltv: legacy_config.manual_ltv,
        fee_rate: legacy_config.fee_rate,
        tax_rate: legacy_config.tax_rate,
    };
    save_config(deps.storage, &config)?;
    Ok(Response::default().add_attributes(vec![
        ("action", "migrate"),
        (
            "nasset_psi_swap_contract_addr",
            &msg.nasset_psi_swap_contract_addr,
        ),
    ]))
}
