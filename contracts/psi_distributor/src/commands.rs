use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Empty, Env, Response, StdError, SubMsg, WasmMsg,
};

use crate::error::ContractError;
use crate::state::{load_aim_ltv, load_config, store_config};
use crate::{state::Config, ContractResult};
use basset_vault::nasset_token_rewards::{
    AnyoneMsg as NAssetTokenRewardsAnyoneMsg, ExecuteMsg as NAssetTokenRewardsExecuteMsg,
};
use basset_vault::querier::query_token_balance;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ExecuteMsg;

pub fn distribute_rewards(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let config: Config = load_config(deps.storage)?;
    let psi_balance: Uint256 =
        query_token_balance(deps.as_ref(), &config.psi_token, &env.contract.address)?.into();

    if psi_balance.is_zero() {
        return Err(StdError::generic_err("psi balance is zero").into());
    }

    let aim_ltv = load_aim_ltv(deps.as_ref(), &config)?;

    let rewards_distribution = RewardsDistribution::calc(
        psi_balance,
        aim_ltv,
        config.manual_ltv,
        config.fee_rate,
        config.tax_rate,
    );

    let mut messages: Vec<SubMsg<Empty>> = Vec::with_capacity(4);
    if !rewards_distribution.nasset_holder.is_zero() {
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.nasset_token_rewards_contract.to_string(),
                amount: rewards_distribution.nasset_holder.into(),
            })?,
        })));

        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            funds: vec![],
            msg: to_binary(&NAssetTokenRewardsExecuteMsg::Anyone {
                anyone_msg: NAssetTokenRewardsAnyoneMsg::UpdateGlobalIndex {},
            })?,
        })));
    }

    if !rewards_distribution.governance.is_zero() {
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.governance_contract.to_string(),
                amount: rewards_distribution.governance.into(),
            })?,
        })));
    }

    if !rewards_distribution.community_pool.is_zero() {
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.psi_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.community_pool_contract.to_string(),
                amount: rewards_distribution.community_pool.into(),
            })?,
        })));
    }

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(vec![
            ("action", "rewards_distribution"),
            (
                "nassest_holder_rewards",
                &rewards_distribution.nasset_holder.to_string(),
            ),
            (
                "governance_rewars",
                &rewards_distribution.governance.to_string(),
            ),
            (
                "community_pool_rewars",
                &rewards_distribution.community_pool.to_string(),
            ),
        ]))
}

struct RewardsDistribution {
    pub nasset_holder: Uint256,
    pub governance: Uint256,
    pub community_pool: Uint256,
}

impl RewardsDistribution {
    pub fn calc(
        psi_amount: Uint256,
        aim_ltv: Decimal256,
        manual_ltv: Decimal256,
        fee_rate: Decimal256,
        tax_rate: Decimal256,
    ) -> Self {
        if manual_ltv >= aim_ltv {
            return Self {
                nasset_holder: psi_amount,
                governance: Uint256::zero(),
                community_pool: Uint256::zero(),
            };
        }

        let protocol_fee = (aim_ltv - manual_ltv) * fee_rate;
        let protocol_rewards = psi_amount * protocol_fee;

        let community_pool_rewards = protocol_rewards * tax_rate;
        let governance_rewars = protocol_rewards - community_pool_rewards;
        let nassest_holder_rewards = psi_amount - protocol_rewards;

        Self {
            nasset_holder: nassest_holder_rewards,
            governance: governance_rewars,
            community_pool: community_pool_rewards,
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    mut current_config: Config,
    governance_contract_addr: Option<String>,
    nasset_token_rewards_contract_addr: Option<String>,
    community_pool_contract_addr: Option<String>,
    basset_vault_strategy_contract_addr: Option<String>,
    manual_ltv: Option<Decimal256>,
    fee_rate: Option<Decimal256>,
    tax_rate: Option<Decimal256>,
) -> ContractResult<Response> {
    if let Some(ref governance_contract_addr) = governance_contract_addr {
        current_config.governance_contract = deps.api.addr_validate(governance_contract_addr)?;
    }

    if let Some(ref nasset_token_rewards_contract_addr) = nasset_token_rewards_contract_addr {
        current_config.nasset_token_rewards_contract =
            deps.api.addr_validate(nasset_token_rewards_contract_addr)?;
    }

    if let Some(ref community_pool_contract_addr) = community_pool_contract_addr {
        current_config.community_pool_contract =
            deps.api.addr_validate(community_pool_contract_addr)?;
    }

    if let Some(ref basset_vault_strategy_contract_addr) = basset_vault_strategy_contract_addr {
        current_config.basset_vault_strategy_contract = deps
            .api
            .addr_validate(basset_vault_strategy_contract_addr)?;
    }

    let one = Decimal256::one();
    if let Some(manual_ltv) = manual_ltv {
        validate_field_to_one(&manual_ltv, "manual_ltv", &one)?;
        current_config.manual_ltv = manual_ltv;
    }

    if let Some(fee_rate) = fee_rate {
        validate_field_to_one(&fee_rate, "fee_rate", &one)?;
        current_config.fee_rate = fee_rate;
    }

    if let Some(tax_rate) = tax_rate {
        validate_field_to_one(&tax_rate, "tax_rate", &one)?;
        current_config.tax_rate = tax_rate;
    }

    store_config(deps.storage, &current_config)?;
    Ok(Response::default())
}

fn validate_field_to_one(
    field_value: &Decimal256,
    field_name: &str,
    one: &Decimal256,
) -> Result<(), ContractError> {
    if field_value >= one {
        return Err(
            StdError::generic_err(format!("'{}' should be lesser than one", field_name)).into(),
        );
    }

    return Ok(());
}

#[cfg(test)]
mod test {
    use super::RewardsDistribution;
    use cosmwasm_bignumber::{Decimal256, Uint256};
    use std::str::FromStr;

    #[test]
    pub fn manual_ltv_bigger_than_aim() {
        let psi_amount = Uint256::from(1_000u64);
        let aim_ltv = Decimal256::from_str("0.8").unwrap();
        let manual_ltv = Decimal256::from_str("0.81").unwrap();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, psi_amount);
        assert_eq!(rewards_distribution.governance, Uint256::zero());
        assert_eq!(rewards_distribution.community_pool, Uint256::zero());
    }

    #[test]
    pub fn manual_ltv_equlas_than_aim() {
        let psi_amount = Uint256::from(1_000u64);
        let aim_ltv = Decimal256::from_str("0.8").unwrap();
        let manual_ltv = Decimal256::from_str("0.8").unwrap();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, psi_amount);
        assert_eq!(rewards_distribution.governance, Uint256::zero());
        assert_eq!(rewards_distribution.community_pool, Uint256::zero());
    }

    #[test]
    pub fn normal_case() {
        let psi_amount = Uint256::from(1_000u64);
        let aim_ltv = Decimal256::from_str("0.8").unwrap();
        let manual_ltv = Decimal256::from_str("0.6").unwrap();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, Uint256::from(900u64));
        assert_eq!(rewards_distribution.governance, Uint256::from(75u64));
        assert_eq!(rewards_distribution.community_pool, Uint256::from(25u64));
    }

    #[test]
    pub fn normal_case_2() {
        let psi_amount = Uint256::from(1_000u64);
        let aim_ltv = Decimal256::from_str("1").unwrap();
        let manual_ltv = Decimal256::zero();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, Uint256::from(500u64));
        assert_eq!(rewards_distribution.governance, Uint256::from(375u64));
        assert_eq!(rewards_distribution.community_pool, Uint256::from(125u64));
    }

    #[test]
    pub fn small_amount_1() {
        let psi_amount = Uint256::from(9u64);
        let aim_ltv = Decimal256::from_str("0.8").unwrap();
        let manual_ltv = Decimal256::from_str("0.6").unwrap();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, Uint256::from(9u64));
        assert_eq!(rewards_distribution.governance, Uint256::zero());
        assert_eq!(rewards_distribution.community_pool, Uint256::zero());
    }

    #[test]
    pub fn small_amount_2() {
        let psi_amount = Uint256::from(10u64);
        let aim_ltv = Decimal256::from_str("0.8").unwrap();
        let manual_ltv = Decimal256::from_str("0.6").unwrap();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, Uint256::from(9u64));
        assert_eq!(rewards_distribution.governance, Uint256::from(1u64));
        assert_eq!(rewards_distribution.community_pool, Uint256::zero());
    }

    #[test]
    pub fn small_amount_3() {
        let psi_amount = Uint256::from(40u64);
        let aim_ltv = Decimal256::from_str("0.8").unwrap();
        let manual_ltv = Decimal256::from_str("0.6").unwrap();
        let fee_rate = Decimal256::from_str("0.5").unwrap();
        let tax_rate = Decimal256::from_str("0.25").unwrap();

        let rewards_distribution =
            RewardsDistribution::calc(psi_amount, aim_ltv, manual_ltv, fee_rate, tax_rate);

        assert_eq!(rewards_distribution.nasset_holder, Uint256::from(36u64));
        assert_eq!(rewards_distribution.governance, Uint256::from(3u64));
        assert_eq!(rewards_distribution.community_pool, Uint256::from(1u64));
    }
}
