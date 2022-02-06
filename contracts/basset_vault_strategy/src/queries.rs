use crate::price::{query_price, PriceResponse};
use basset_vault::anchor::borrow_apr::BorrowNetApr;
use basset_vault::basset_vault_strategy::{BorrowerActionResponse, ConfigResponse};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdError, StdResult, Timestamp};

use crate::state::{load_config, Config};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        anchor_market_contract: config.anchor_market_contract.to_string(),
        anchor_interest_model_contract: config.anchor_interest_model_contract.to_string(),
        anchor_overseer_contract: config.anchor_overseer_contract.to_string(),
        governance_contract: config.governance_contract.to_string(),
        oracle_contract: config.oracle_contract.to_string(),
        basset_token: config.basset_token.to_string(),
        stable_denom: config.stable_denom.clone(),
        borrow_ltv_max: config.get_borrow_ltv_max(),
        borrow_ltv_min: config.get_borrow_ltv_min(),
        borrow_ltv_aim: config.get_borrow_ltv_aim(),
        basset_max_ltv: config.get_basset_max_ltv(),
        buffer_part: config.get_buffer_part(),
        price_timeframe: config.price_timeframe,
    })
}

struct LTVInfo {
    basset_price: Decimal256,
    borrow_ltv_max: Decimal256,
    borrow_ltv_min: Decimal256,
    borrow_ltv_aim: Decimal256,
}

impl LTVInfo {
    pub fn new(
        borrow_ltv_max: Decimal256,
        borrow_ltv_min: Decimal256,
        borrow_ltv_aim: Decimal256,
        price_timeframe: u64,
        price: &PriceResponse,
        block_time: Timestamp,
    ) -> Self {
        let valid_update_time = block_time.seconds() - price_timeframe;
        if price.last_updated_base < valid_update_time
            || price.last_updated_quote < valid_update_time
        {
            //if price is too old we divide our LTV by half to avoid sharp liquidation
            return Self {
                basset_price: price.rate,
                borrow_ltv_max: borrow_ltv_max * Decimal256::percent(50),
                borrow_ltv_min: borrow_ltv_min * Decimal256::percent(50),
                borrow_ltv_aim: borrow_ltv_aim * Decimal256::percent(50),
            };
        }

        return Self {
            basset_price: price.rate,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        };
    }
}

struct AnchorApr {
    earn: Decimal256,
    borrow: BorrowNetApr,
}

fn query_anchor_apr(deps: Deps, config: &Config) -> StdResult<AnchorApr> {
    let earn = basset_vault::anchor::earn_apr::query_anchor_earn_apr(
        deps,
        &config.anchor_overseer_contract,
    )?;

    let borrow = basset_vault::anchor::borrow_apr::query_anchor_borrow_net_apr(
        deps,
        &config.anchor_market_contract,
        &config.anchor_interest_model_contract,
        config.anchor_token.clone(),
        &config.anc_ust_swap_contract,
        config.stable_denom.clone(),
    )?;

    Ok(AnchorApr { earn, borrow })
}

impl AnchorApr {
    fn is_profitable(&self, buffer_part: Decimal256) -> bool {
        let deposited_part = Decimal256::one() - buffer_part;
        let earn_of_deposited_part = self.earn * deposited_part;

        let distr = self.borrow.distribution_apr;
        let interest = self.borrow.interest_apr;

        // The ideal formulae would be:
        //
        // borrow_apr = distr_apr - interest_apr
        // apr = earn_apr + borrow_apr
        // anchor_has_profit = apr > 0
        //
        // But `Decimal256` does not support negative numbers, so we do the following hacks:
        //
        // anchor_has_profit = distribution_apr + borrow_apr - interest_apr > 0
        //                   = distribution_apr + borrow_apr > interest_apr
        distr + earn_of_deposited_part > interest
    }
}

pub fn borrower_action(
    deps: Deps,
    env: Env,
    basset_in_contract_address: Uint256,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
) -> StdResult<BorrowerActionResponse> {
    let config: Config = load_config(deps.storage)?;

    let oracle_price: PriceResponse = query_price(
        deps,
        &config.oracle_contract,
        config.basset_token.to_string(),
        config.stable_denom.to_string(),
    )?;

    let ltv_info = LTVInfo::new(
        config.get_borrow_ltv_max(),
        config.get_borrow_ltv_min(),
        config.get_borrow_ltv_aim(),
        config.price_timeframe,
        &oracle_price,
        env.block.time,
    );

    let anchor_apr = query_anchor_apr(deps, &config);

    let buffer_part = config.get_buffer_part();

    if buffer_part > Decimal256::one() {
        return Err(StdError::generic_err(
            "Buffer part must be less than or equal to 1.0",
        ));
    }

    let response = calc_borrower_action(
        anchor_apr,
        ltv_info,
        basset_in_contract_address,
        borrowed_amount,
        locked_basset_amount,
        config.get_basset_max_ltv(),
        config.get_buffer_part(),
    );

    Ok(response)
}

fn calc_borrower_action(
    anchor_apr: StdResult<AnchorApr>,
    ltv_info: LTVInfo,
    basset_in_contract_address: Uint256,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
    basset_max_ltv: Decimal256,
    buffer_part: Decimal256,
) -> BorrowerActionResponse {
    let anchor_has_profit = anchor_apr
        .map(|a| a.is_profitable(buffer_part))
        .unwrap_or(false); // TODO: fallback option, withdraw or not?

    if !anchor_has_profit {
        // Withdraw all if there are anything to withdraw
        if locked_basset_amount != Uint256::zero() {
            // If the actual `locked_basset_amount`
            // is more than provided `locked_basset_amount`,
            // withdraw logic still repay all actual borrowed amount
            return BorrowerActionResponse::withdraw_all(locked_basset_amount);
        } else {
            return BorrowerActionResponse::nothing();
        }
    }

    if basset_in_contract_address != Uint256::zero() {
        let action_after = calc_borrower_action_on_profitable_anchor(
            ltv_info,
            borrowed_amount,
            basset_in_contract_address, // locked_basset_amount after deposit,
            basset_max_ltv,
            buffer_part,
        );
        BorrowerActionResponse::deposit(basset_in_contract_address, action_after)
    } else {
        calc_borrower_action_on_profitable_anchor(
            ltv_info,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        )
    }
}

fn calc_borrower_action_on_profitable_anchor(
    ltv_info: LTVInfo,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
    basset_max_ltv: Decimal256,
    buffer_part: Decimal256,
) -> BorrowerActionResponse {
    //repay loan if you can't manage it
    if ltv_info.basset_price == Decimal256::zero() || locked_basset_amount == Uint256::zero() {
        if borrowed_amount > Uint256::zero() {
            return BorrowerActionResponse::repay(borrowed_amount, Uint256::zero());
        } else {
            return BorrowerActionResponse::nothing();
        }
    }

    let max_borrow_amount: Uint256 = locked_basset_amount * ltv_info.basset_price * basset_max_ltv;
    // If locked_basset_amount is small enough and basset_price is low It's possible max_borrow_amount = 0
    if max_borrow_amount == Uint256::zero() {
        if borrowed_amount > Uint256::zero() {
            return BorrowerActionResponse::repay(borrowed_amount, Uint256::zero());
        } else {
            return BorrowerActionResponse::nothing();
        }
    }
    let current_ltv: Decimal256 =
        Decimal256::from_uint256(borrowed_amount) / Decimal256::from_uint256(max_borrow_amount);

    let buffer_size = max_borrow_amount * buffer_part;
    let aim_borrow_amount = ltv_info.borrow_ltv_aim * max_borrow_amount;
    if current_ltv >= ltv_info.borrow_ltv_max {
        let repay_amount = borrowed_amount - aim_borrow_amount;
        BorrowerActionResponse::repay(repay_amount, buffer_size)
    } else if current_ltv <= ltv_info.borrow_ltv_min
        && aim_borrow_amount != Uint256::zero()
        && aim_borrow_amount > borrowed_amount
    {
        let borrow_amount = aim_borrow_amount - borrowed_amount;
        BorrowerActionResponse::borrow(borrow_amount, buffer_size)
    } else {
        BorrowerActionResponse::nothing()
    }
}

#[cfg(feature = "integration_tests_build")]
pub mod test_anchor_apr_calculation {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct AnchorApr {
        pub anchor_earn_apr: Decimal256,
        pub anchor_borrow_distribution_apr: Decimal256,
        pub anchor_borrow_interest_apr: Decimal256,
    }

    pub fn calculate(deps: Deps, _env: Env) -> StdResult<AnchorApr> {
        let config: Config = load_config(deps.storage)?;

        let anchor_earn_apr = basset_vault::anchor::earn_apr::query_anchor_earn_apr(
            deps,
            &config.anchor_overseer_contract,
        )?;

        let anchor_borrow_apr = basset_vault::anchor::borrow_apr::query_anchor_borrow_net_apr(
            deps,
            &config.anchor_market_contract,
            &config.anchor_interest_model_contract,
            config.anchor_token.clone(),
            &config.anc_ust_swap_contract,
            config.stable_denom.clone(),
        )?;

        Ok(AnchorApr {
            anchor_earn_apr,
            anchor_borrow_distribution_apr: anchor_borrow_apr.distribution_apr,
            anchor_borrow_interest_apr: anchor_borrow_apr.interest_apr,
        })
    }
}

#[cfg(test)]
mod test {
    use basset_vault::basset_vault_strategy::BorrowerActionResponse;
    use basset_vault::anchor::borrow_apr::BorrowNetApr;
    use cosmwasm_bignumber::{Decimal256, Uint256};
    use cosmwasm_std::Timestamp;
    use std::str::FromStr;

    use crate::{price::PriceResponse, queries::LTVInfo};

    use super::{calc_borrower_action, AnchorApr};

    #[test]
    fn repay_loan() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(519_750u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("5.5").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 519_750 / 577_500 = 0.9
        //to_repay = (0.9 - 0.8) * 577_500 = 57_750
        //buffer_size = 0.018 * 577_500 = 10_395
        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(Uint256::from(57_750u64), Uint256::from(10_395u64))
        );
    }

    #[test]
    fn borrow_more() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(346_500u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("5.5").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 346_500 / 577_500 = 0.6
        //to_borrow =  (0.8 - 0.6) * 577_500 = 115_500
        //buffer_size = 0.018 * 577_500 = 10_395
        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::borrow(Uint256::from(115_500u64), Uint256::from(10_395u64))
        );
    }

    #[test]
    fn nothing_on_aim_borrow_amount_equals_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::zero();
        let locked_basset_amount = Uint256::from(3u64);
        let basset_max_ltv = Decimal256::from_str("0.6").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("1").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }

    #[test]
    fn do_nothing() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(473_550u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("5.5").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 473_550 / 577_500 = 0.82
        //0.75 < 0.82 < 0.85 => do nothing
        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }

    #[test]
    fn locked_amount_is_zero_and_borrowed_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::zero();
        let locked_basset_amount = Uint256::zero();
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("5.5").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }

    #[test]
    fn locked_amount_is_zero_and_borrowed_not_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(473_550u64);
        let locked_basset_amount = Uint256::zero();
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();

        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("5.5").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };
        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(borrowed_amount, Uint256::zero())
        );
    }

    #[test]
    fn collateral_value_is_low_and_borrowed_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::zero();
        let locked_basset_amount = Uint256::from(1u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("1").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }

    #[test]
    fn collateral_value_is_low_and_borrowed_not_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(10_000u64);
        let locked_basset_amount = Uint256::from(1u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("1").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(borrowed_amount, Uint256::zero())
        );
    }

    #[test]
    fn borrowed_amount_is_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::zero();
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("5.5").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 0.0
        //to_borrow = (0.8 - 0.0) * 577_500 = 462_000
        //buffer_size = 0.018 * 577_500 = 10_395
        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::borrow(Uint256::from(462_000u64), Uint256::from(10_395u64))
        );
    }

    #[test]
    fn asset_price_is_zero() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(473_550u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::zero(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(borrowed_amount, Uint256::zero())
        );
    }

    #[test]
    fn price_is_too_old() {
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();
        let block_time = Timestamp::from_seconds(100);

        //base is old
        {
            let price_response = PriceResponse {
                rate: Decimal256::one(),
                last_updated_base: 40,
                last_updated_quote: 80,
            };

            let ltv_info = LTVInfo::new(
                borrow_ltv_max,
                borrow_ltv_min,
                borrow_ltv_aim,
                50,
                &price_response,
                block_time,
            );

            assert_eq!(
                Decimal256::from_str("0.425").unwrap(),
                ltv_info.borrow_ltv_max
            );
            assert_eq!(
                Decimal256::from_str("0.375").unwrap(),
                ltv_info.borrow_ltv_min
            );
            assert_eq!(
                Decimal256::from_str("0.4").unwrap(),
                ltv_info.borrow_ltv_aim
            );
        }

        //quote is old
        {
            let price_response = PriceResponse {
                rate: Decimal256::one(),
                last_updated_base: 80,
                last_updated_quote: 40,
            };

            let ltv_info = LTVInfo::new(
                borrow_ltv_max,
                borrow_ltv_min,
                borrow_ltv_aim,
                50,
                &price_response,
                block_time,
            );

            assert_eq!(
                Decimal256::from_str("0.425").unwrap(),
                ltv_info.borrow_ltv_max
            );
            assert_eq!(
                Decimal256::from_str("0.375").unwrap(),
                ltv_info.borrow_ltv_min
            );
            assert_eq!(
                Decimal256::from_str("0.4").unwrap(),
                ltv_info.borrow_ltv_aim
            );
        }
    }

    #[test]
    fn price_is_fresh() {
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        let block_time = Timestamp::from_seconds(100);
        let price_response = PriceResponse {
            rate: Decimal256::one(),
            last_updated_base: 80,
            last_updated_quote: 80,
        };

        let ltv_info = LTVInfo::new(
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
            50,
            &price_response,
            block_time,
        );

        assert_eq!(borrow_ltv_max, ltv_info.borrow_ltv_max);
        assert_eq!(borrow_ltv_min, ltv_info.borrow_ltv_min);
        assert_eq!(borrow_ltv_aim, ltv_info.borrow_ltv_aim);
    }

    #[test]
    fn loan_equals_to_aim_borrow_amount() {
        let anchor_apr = AnchorApr {
            earn: Decimal256::from_str("0.20").unwrap(),
            borrow: BorrowNetApr {
                distribution_apr: Decimal256::from_str("0.10").unwrap(),
                interest_apr: Decimal256::from_str("0.05").unwrap(),
            },
        };
        let basset_in_contract_address = Uint256::zero();
        let borrowed_amount = Uint256::from(48u64);
        let locked_basset_amount = Uint256::from(100u64);
        let basset_max_ltv = Decimal256::from_str("0.6").unwrap();
        let buffer_part = Decimal256::from_str("0.018").unwrap();
        let ltv_info = LTVInfo {
            basset_price: Decimal256::from_str("1").unwrap(),
            borrow_ltv_max: Decimal256::from_str("0.85").unwrap(),
            borrow_ltv_min: Decimal256::from_str("0.75").unwrap(),
            borrow_ltv_aim: Decimal256::from_str("0.8").unwrap(),
        };

        // aim_borrow_amount = locked_basset_amount * basset_price * basset_max_ltv * borrow_ltv_aim
        // aim_borrow_amount = 100 * 1 * 0,6 * 0,8 = 48 (= borrowed_amount = 48)

        let borrower_action = calc_borrower_action(
            Ok(anchor_apr),
            ltv_info,
            basset_in_contract_address,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            buffer_part,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }
}
