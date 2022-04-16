use basset_vault::{
    anchor::basset_custody::get_basset_in_custody,
    anchor::market::{
        query_borrower_info, query_market_config, query_market_state, BorrowerInfoResponse,
        ConfigResponse as AnchorMarketConfigResponse, StateResponse as AnchorMarketStateResponse,
    },
    basset_vault::{
        ChildContractsInfoResponse, ConfigResponse, IsRewardsClaimableResponse, RebalanceResponse,
    },
    basset_vault_strategy::{query_borrower_action, BorrowerActionResponse},
    querier::{query_balance, query_token_balance},
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdResult};

use crate::state::{load_child_contracts_info, load_config};
use crate::{state::Config, utils::is_anc_rewards_claimable};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        governance_contract: config.governance_contract.to_string(),
        anchor_token_addr: config.anchor_token.to_string(),
        anchor_overseer_contract_addr: config.anchor_overseer_contract.to_string(),
        anchor_market_contract_addr: config.anchor_market_contract.to_string(),
        anchor_custody_basset_contract_addr: config.anchor_custody_basset_contract.to_string(),
        anchor_basset_reward_addr: config.anchor_basset_reward_contract.to_string(),
        anc_stable_swap_contract_addr: config.anc_stable_swap_contract.to_string(),
        psi_stable_swap_contract_addr: config.psi_stable_swap_contract.to_string(),
        nasset_token_addr: config.nasset_token.to_string(),
        basset_token_addr: config.basset_token.to_string(),
        aterra_token_addr: config.aterra_token.to_string(),
        psi_token_addr: config.psi_token.to_string(),
        basset_vault_strategy_contract_addr: config.basset_vault_strategy_contract.to_string(),
        stable_denom: config.stable_denom,
        claiming_rewards_delay: config.claiming_rewards_delay,
        over_loan_balance_value: config.over_loan_balance_value,
        psi_distributor_addr: config.psi_distributor.to_string(),
    })
}

pub fn query_rebalance(deps: Deps, env: Env) -> StdResult<RebalanceResponse> {
    let config: Config = load_config(deps.storage)?;

    let basset_in_contract_address = query_token_balance(
        deps,
        &config.basset_token,
        &env.contract.address
    );

    // basset balance in custody contract
    let basset_in_custody = get_basset_in_custody(
        deps,
        &config.anchor_custody_basset_contract,
        &env.contract.address,
    )?;

    let borrower_info: BorrowerInfoResponse =
        query_borrower_info(deps, &config.anchor_market_contract, &env.contract.address)?;
    let borrowed_ust = borrower_info.loan_amount;

    let borrower_action = query_borrower_action(
        deps,
        &config.basset_vault_strategy_contract,
        basset_in_contract_address.into(),
        borrowed_ust,
        basset_in_custody,
    )?;

    borrower_action_to_response(borrower_action, deps, env, &config)
}

fn borrower_action_to_response(
    action: BorrowerActionResponse,
    deps: Deps,
    env: Env,
    config: &Config
) -> StdResult<RebalanceResponse> {
    let response = match action {
        BorrowerActionResponse::Nothing {} => RebalanceResponse::Nothing {},
        BorrowerActionResponse::Repay {
            amount,
            advised_buffer_size,
        } => RebalanceResponse::Repay {
            amount,
            advised_buffer_size,
        },
        BorrowerActionResponse::Borrow {
            amount,
            advised_buffer_size,
        } => {
            let anchor_market_state = query_market_state(deps, &config.anchor_market_contract)?;
            let anchor_market_balance = query_balance(
                &deps.querier,
                &config.anchor_market_contract,
                config.stable_denom.clone(),
            )?;
            let anchor_market_config = query_market_config(deps, &config.anchor_market_contract)?;
            let is_borrowing_possible = assert_max_borrow_factor(
                anchor_market_config,
                anchor_market_state,
                anchor_market_balance.into(),
                amount,
            );

            RebalanceResponse::Borrow {
                amount,
                advised_buffer_size,
                is_possible: is_borrowing_possible,
            }
        },
        BorrowerActionResponse::Deposit { deposit_amount, action_after } => RebalanceResponse::Deposit {
            deposit_amount,
            action_after: Box::new(borrower_action_to_response(*action_after, deps, env, config)?)
        },
        BorrowerActionResponse::RepayAllAndWithdraw { withdraw_amount } => RebalanceResponse::RepayAllAndWithdraw { withdraw_amount },
    };
    Ok(response)
}

//copypasted from anchor_market contract
fn assert_max_borrow_factor(
    market_config: AnchorMarketConfigResponse,
    market_state: AnchorMarketStateResponse,
    market_balance: Uint256,
    borrow_amount: Uint256,
) -> bool {
    let current_balance = Decimal256::from_uint256(market_balance);
    let borrow_amount = Decimal256::from_uint256(borrow_amount);

    // Assert max borrow factor
    if market_state.total_liabilities + borrow_amount
        > (current_balance + market_state.total_liabilities - market_state.total_reserves)
            * market_config.max_borrow_factor
    {
        return false;
    }

    // Assert available balance
    if borrow_amount + market_state.total_reserves > current_balance {
        return false;
    }

    true
}

pub fn child_contracts_code_id(deps: Deps) -> StdResult<ChildContractsInfoResponse> {
    let child_contracts_info = load_child_contracts_info(deps.storage)?;
    Ok(ChildContractsInfoResponse {
        nasset_token_code_id: child_contracts_info.nasset_token_code_id,
        nasset_token_rewards_code_id: child_contracts_info.nasset_token_rewards_code_id,
        psi_distributor_code_id: child_contracts_info.psi_distributor_code_id,
        collateral_token_symbol: child_contracts_info.collateral_token_symbol,
        community_pool_contract_addr: child_contracts_info.community_pool_contract_addr,
        manual_ltv: child_contracts_info.manual_ltv,
        fee_rate: child_contracts_info.fee_rate,
        tax_rate: child_contracts_info.tax_rate,
    })
}

pub fn is_rewards_claimable(deps: Deps, env: Env) -> StdResult<IsRewardsClaimableResponse> {
    let config: Config = load_config(deps.storage)?;
    let borrower_info =
        query_borrower_info(deps, &config.anchor_market_contract, &env.contract.address)?;

    let is_rewards_claimable = is_anc_rewards_claimable(borrower_info.pending_rewards);

    Ok(IsRewardsClaimableResponse {
        claimable: is_rewards_claimable,
        anc_amount: borrower_info.pending_rewards,
        last_claiming_height: 0, //legacy field
        current_height: 0,       //legacy field
    })
}
