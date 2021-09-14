use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, StdResult, Storage};

use crate::{error::ContractError, ContractResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub governance_contract: Addr,
    pub oracle_contract: Addr,
    pub basset_token: Addr,
    pub stable_denom: String,
    borrow_ltv_max: Decimal256,
    borrow_ltv_min: Decimal256,
    borrow_ltv_aim: Decimal256,
    basset_max_ltv: Decimal256,
    //(max_ltv - aim_ltv)*0.35
    //(0.85-0.8) * 0.35 = 0.018
    //to be able to repay loan in 3 iterations (in case of aterra locked)
    buffer_part: Decimal256,
    pub price_timeframe: u64,
}

impl Config {
    pub fn new(
        governance_contract: Addr,
        oracle_contract: Addr,
        basset_token: Addr,
        stable_denom: String,
        borrow_ltv_max: Decimal256,
        borrow_ltv_min: Decimal256,
        borrow_ltv_aim: Decimal256,
        basset_max_ltv: Decimal256,
        buffer_part: Decimal256,
        price_timeframe: u64,
    ) -> ContractResult<Self> {
        Self::validate_borrow_ltvs(borrow_ltv_max, borrow_ltv_min, borrow_ltv_aim)?;

        let mut config = Config {
            governance_contract,
            oracle_contract,
            basset_token,
            stable_denom,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
            basset_max_ltv,
            buffer_part,
            price_timeframe,
        };

        config.set_basset_max_ltv(basset_max_ltv)?;
        config.set_buffer_part(buffer_part)?;

        Ok(config)
    }

    pub fn set_basset_max_ltv(&mut self, value: Decimal256) -> ContractResult<()> {
        if value.is_zero() || value > Decimal256::one() {
            return Err(ContractError::InappropriateValue);
        }

        self.basset_max_ltv = value;
        Ok(())
    }

    pub fn set_buffer_part(&mut self, value: Decimal256) -> ContractResult<()> {
        if value.is_zero() || value > Decimal256::one() {
            return Err(ContractError::InappropriateValue);
        }

        self.buffer_part = value;
        Ok(())
    }

    pub fn validate_and_set_borrow_ltvs(
        &mut self,
        borrow_ltv_max: Decimal256,
        borrow_ltv_min: Decimal256,
        borrow_ltv_aim: Decimal256,
    ) -> ContractResult<()> {
        Self::validate_borrow_ltvs(borrow_ltv_max, borrow_ltv_min, borrow_ltv_aim)?;

        self.borrow_ltv_max = borrow_ltv_max;
        self.borrow_ltv_min = borrow_ltv_min;
        self.borrow_ltv_aim = borrow_ltv_aim;

        Ok(())
    }

    fn validate_borrow_ltvs(
        borrow_ltv_max: Decimal256,
        borrow_ltv_min: Decimal256,
        borrow_ltv_aim: Decimal256,
    ) -> ContractResult<()> {
        let one = Decimal256::one();
        let zero = Decimal256::zero();
        if one >= borrow_ltv_max
            && borrow_ltv_max > borrow_ltv_aim
            && borrow_ltv_aim > borrow_ltv_min
            && borrow_ltv_min >= zero
        {
            Ok(())
        } else {
            Err(ContractError::InappropriateValue)
        }
    }

    pub fn get_borrow_ltv_max(&self) -> Decimal256 {
        self.borrow_ltv_max
    }

    pub fn get_borrow_ltv_min(&self) -> Decimal256 {
        self.borrow_ltv_min
    }

    pub fn get_borrow_ltv_aim(&self) -> Decimal256 {
        self.borrow_ltv_aim
    }

    pub fn get_basset_max_ltv(&self) -> Decimal256 {
        self.basset_max_ltv
    }

    pub fn get_buffer_part(&self) -> Decimal256 {
        self.buffer_part
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GovernanceUpdateState {
    pub new_governance_contract_addr: Addr,
    pub wait_approve_until: u64,
}

static KEY_CONFIG: Item<Config> = Item::new("config");
static KEY_GOVERNANCE_UPDATE: Item<GovernanceUpdateState> = Item::new("gov_update");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    KEY_CONFIG.load(storage)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    KEY_CONFIG.save(storage, config)
}

pub fn load_gov_update(storage: &dyn Storage) -> StdResult<GovernanceUpdateState> {
    KEY_GOVERNANCE_UPDATE.load(storage)
}

pub fn store_gov_update(
    storage: &mut dyn Storage,
    gov_update: &GovernanceUpdateState,
) -> StdResult<()> {
    KEY_GOVERNANCE_UPDATE.save(storage, gov_update)
}

pub fn remove_gov_update(storage: &mut dyn Storage) -> () {
    KEY_GOVERNANCE_UPDATE.remove(storage)
}

#[cfg(test)]
mod test {
    use super::Config;
    use cosmwasm_bignumber::Decimal256;
    use cosmwasm_std::Addr;
    use std::str::FromStr;

    const PRICE_TIMEFRAME: u64 = 60;
    #[test]
    pub fn fail_to_initiate_with_wrong_values() {
        // max = aim
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.75").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("0.018").unwrap(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());

        // min = aim
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.85").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("0.018").unwrap(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());

        // min > max
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.85").unwrap(),
            Decimal256::from_str("0.9").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("0.018").unwrap(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());

        // max < min
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.4").unwrap(),
            Decimal256::from_str("0.6").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("0.018").unwrap(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());

        // max > 1
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("1.4").unwrap(),
            Decimal256::from_str("0.6").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("0.018").unwrap(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());

        // buffer > 1
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.9").unwrap(),
            Decimal256::from_str("0.6").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("1.1").unwrap(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());

        // buffer = 0
        let creation_res = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.9").unwrap(),
            Decimal256::from_str("0.6").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::zero(),
            PRICE_TIMEFRAME,
        );
        assert!(creation_res.is_err());
    }

    #[test]
    pub fn fail_to_update_with_wrong_values() {
        let mut config = Config::new(
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            Addr::unchecked("addr0001"),
            "uust".to_string(),
            Decimal256::from_str("0.9").unwrap(),
            Decimal256::from_str("0.6").unwrap(),
            Decimal256::from_str("0.8").unwrap(),
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::from_str("0.018").unwrap(),
            PRICE_TIMEFRAME,
        )
        .unwrap();

        assert!(config.set_basset_max_ltv(Decimal256::zero()).is_err());
        assert!(config
            .set_basset_max_ltv(Decimal256::from_str("1.5").unwrap())
            .is_err());

        assert!(config.set_buffer_part(Decimal256::zero()).is_err());
        assert!(config
            .set_buffer_part(Decimal256::from_str("1.5").unwrap())
            .is_err());

        let update_res = config.validate_and_set_borrow_ltvs(
            Decimal256::from_str("1.5").unwrap(),
            config.get_borrow_ltv_min(),
            config.get_borrow_ltv_aim(),
        );
        assert!(update_res.is_err());
        let update_res = config.validate_and_set_borrow_ltvs(
            Decimal256::from_str("0.5").unwrap(),
            config.get_borrow_ltv_min(),
            config.get_borrow_ltv_aim(),
        );
        assert!(update_res.is_err());
        let update_res = config.validate_and_set_borrow_ltvs(
            Decimal256::from_str("0.8").unwrap(),
            config.get_borrow_ltv_min(),
            config.get_borrow_ltv_aim(),
        );
        assert!(update_res.is_err());

        let update_res = config.validate_and_set_borrow_ltvs(
            config.get_borrow_ltv_max(),
            Decimal256::from_str("0.8").unwrap(),
            config.get_borrow_ltv_aim(),
        );
        assert!(update_res.is_err());
        let update_res = config.validate_and_set_borrow_ltvs(
            config.get_borrow_ltv_max(),
            Decimal256::from_str("0.95").unwrap(),
            config.get_borrow_ltv_aim(),
        );
        assert!(update_res.is_err());

        let update_res = config.validate_and_set_borrow_ltvs(
            config.get_borrow_ltv_max(),
            config.get_borrow_ltv_min(),
            Decimal256::from_str("0.95").unwrap(),
        );
        assert!(update_res.is_err());
        let update_res = config.validate_and_set_borrow_ltvs(
            config.get_borrow_ltv_max(),
            config.get_borrow_ltv_min(),
            Decimal256::from_str("0.5").unwrap(),
        );
        assert!(update_res.is_err());
    }
}
