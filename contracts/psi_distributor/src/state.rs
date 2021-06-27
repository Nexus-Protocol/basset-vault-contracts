use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub nasset_token_addr: Addr,
    pub governance_addr: Addr,
    pub rewards_distribution: RewardsDistribution,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardShare {
    pub recipient: Addr,
    pub share: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsDistribution {
    distribution: Vec<RewardShare>,
}

impl RewardsDistribution {
    pub fn new(rewards_distribution: Vec<RewardShare>) -> StdResult<RewardsDistribution> {
        let total_percentage: Decimal256 = rewards_distribution
            .iter()
            .map(|rs| rs.share)
            .fold(Decimal256::zero(), |sum, share| sum + share);
        if total_percentage != Decimal256::one() {
            return Err(StdError::generic_err(format!(
                "wrong rewards distribution, total sum should be one, but equals {}",
                total_percentage
            )));
        }

        Ok(RewardsDistribution {
            distribution: rewards_distribution,
        })
    }

    pub fn distribution(&self) -> &Vec<RewardShare> {
        &self.distribution
    }
}

const CONFIG: Item<Config> = Item::new("config");

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

#[cfg(test)]
mod test {
    use cosmwasm_bignumber::Decimal256;
    use cosmwasm_std::Addr;

    use super::{RewardShare, RewardsDistribution};

    #[test]
    pub fn zero_shares() {
        let rewards_distribution = RewardsDistribution::new(vec![]);
        assert!(rewards_distribution.is_err())
    }

    #[test]
    pub fn one_element() {
        let one_share = RewardShare {
            recipient: Addr::unchecked("xxx"),
            share: Decimal256::percent(100),
        };
        let rewards_distribution = RewardsDistribution::new(vec![one_share]);
        assert!(rewards_distribution.is_ok())
    }

    #[test]
    pub fn wrong_total_distribution_1() {
        let rewards = vec![
            RewardShare {
                recipient: Addr::unchecked("uuu"),
                share: Decimal256::percent(10),
            },
            RewardShare {
                recipient: Addr::unchecked("xxx"),
                share: Decimal256::percent(15),
            },
        ];
        let rewards_distribution = RewardsDistribution::new(rewards);
        assert!(rewards_distribution.is_err())
    }

    #[test]
    pub fn wrong_total_distribution_2() {
        let rewards = vec![
            RewardShare {
                recipient: Addr::unchecked("uuu"),
                share: Decimal256::percent(10),
            },
            RewardShare {
                recipient: Addr::unchecked("xxx"),
                share: Decimal256::percent(1125),
            },
        ];
        let rewards_distribution = RewardsDistribution::new(rewards);
        assert!(rewards_distribution.is_err())
    }

    #[test]
    pub fn right_total_distribution() {
        let rewards = vec![
            RewardShare {
                recipient: Addr::unchecked("uuu"),
                share: Decimal256::percent(10),
            },
            RewardShare {
                recipient: Addr::unchecked("xxx"),
                share: Decimal256::percent(90),
            },
        ];
        let rewards_distribution = RewardsDistribution::new(rewards);
        assert!(rewards_distribution.is_ok())
    }
}
