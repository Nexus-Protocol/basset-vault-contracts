use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdResult};
use yield_optimizer::{
    basset_farmer::{
        ChildContractsCodeIdResponse, ConfigResponse, IsRewardsClaimableResponse, RebalanceResponse,
    },
    basset_farmer_config::{query_borrower_action, BorrowerActionResponse},
    querier::{
        get_basset_in_custody, query_balance, query_borrower_info, query_market_config,
        query_market_state, AnchorMarketConfigResponse, AnchorMarketStateResponse,
        BorrowerInfoResponse,
    },
};

use crate::state::{load_child_contracts_code_id, load_config};
use crate::{
    state::{load_last_rewards_claiming_height, Config},
    utils::is_anc_rewards_claimable,
};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        governance_contract: config.governance_contract.to_string(),
        anchor_token: config.anchor_token.to_string(),
        anchor_overseer_contract: config.anchor_overseer_contract.to_string(),
        anchor_market_contract: config.anchor_market_contract.to_string(),
        custody_basset_contract: config.anchor_custody_basset_contract.to_string(),
        anc_stable_swap_contract: config.anc_stable_swap_contract.to_string(),
        psi_stable_swap_contract: config.psi_stable_swap_contract.to_string(),
        nasset_token: config.nasset_token.to_string(),
        basset_token: config.basset_token.to_string(),
        aterra_token: config.aterra_token.to_string(),
        psi_token: config.psi_token.to_string(),
        basset_farmer_config_contract: config.basset_farmer_config_contract.to_string(),
        stable_denom: config.stable_denom,
        claiming_rewards_delay: config.claiming_rewards_delay,
    })
}

pub fn query_rebalance(deps: Deps, env: Env) -> StdResult<RebalanceResponse> {
    let config: Config = load_config(deps.storage)?;

    // basset balance in custody contract
    let basset_in_custody = get_basset_in_custody(
        deps,
        &config.anchor_custody_basset_contract,
        &env.contract.address.clone(),
    )?;

    let borrower_info: BorrowerInfoResponse =
        query_borrower_info(deps, &config.anchor_market_contract, &env.contract.address)?;
    let borrowed_ust = borrower_info.loan_amount;

    let borrower_action = query_borrower_action(
        deps,
        &config.basset_farmer_config_contract,
        borrowed_ust,
        basset_in_custody,
    )?;

    let response = match borrower_action {
        BorrowerActionResponse::Nothing => RebalanceResponse::Nothing,
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
                config.stable_denom,
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
        }
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

    return true;
}

pub fn child_contracts_code_id(deps: Deps) -> StdResult<ChildContractsCodeIdResponse> {
    let child_contracts_code_id = load_child_contracts_code_id(deps.storage)?;
    Ok(ChildContractsCodeIdResponse {
        nasset_token: child_contracts_code_id.nasset_token,
        nasset_staker: child_contracts_code_id.nasset_staker,
        psi_distributor: child_contracts_code_id.psi_distributor,
    })
}

pub fn is_rewards_claimable(deps: Deps, env: Env) -> StdResult<IsRewardsClaimableResponse> {
    let config: Config = load_config(deps.storage)?;
    let last_rewards_claiming_height = load_last_rewards_claiming_height(deps.storage)?;
    let current_height = env.block.height;
    let borrower_info =
        query_borrower_info(deps, &config.anchor_market_contract, &env.contract.address)?;

    let is_rewards_claimable = is_anc_rewards_claimable(
        current_height,
        last_rewards_claiming_height,
        config.claiming_rewards_delay,
    );

    Ok(IsRewardsClaimableResponse {
        claimable: is_rewards_claimable,
        anc_amount: borrower_info.pending_rewards,
        last_claiming_height: last_rewards_claiming_height,
        current_height,
    })
}
