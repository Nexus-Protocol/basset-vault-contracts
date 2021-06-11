use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, StdResult};
use yield_optimizer::{
    basset_farmer_config::{BorrowerActionResponse, ConfigResponse},
    querier::{query_price, PriceResponse},
};

use crate::state::{load_config, Config};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = load_config(deps.storage)?;
    Ok(ConfigResponse {
        governance_contract_addr: config.governance_contract_addr,
        oracle_addr: config.oracle_addr,
        basset_token_addr: config.basset_token_addr,
        stable_denom: config.stable_denom,
        borrow_ltv_max: config.borrow_ltv_max,
        borrow_ltv_min: config.borrow_ltv_min,
        borrow_ltv_aim: config.borrow_ltv_aim,
        basset_max_ltv: config.basset_max_ltv,
    })
}

pub fn borrower_action(
    deps: Deps,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
) -> StdResult<BorrowerActionResponse> {
    let config: Config = load_config(deps.storage)?;

    let price: PriceResponse = query_price(
        deps,
        &config.oracle_addr,
        config.basset_token_addr.to_string(),
        config.stable_denom.to_string(),
        None,
    )?;

    let response = calc_borrower_action(
        price.rate,
        borrowed_amount,
        locked_basset_amount,
        config.basset_max_ltv,
        config.borrow_ltv_max,
        config.borrow_ltv_min,
        config.borrow_ltv_aim,
    );
    Ok(response)
}

fn calc_borrower_action(
    basset_price: Decimal256,
    borrowed_amount: Uint256,
    locked_basset_amount: Uint256,
    basset_max_ltv: Decimal256,
    borrow_ltv_max: Decimal256,
    borrow_ltv_min: Decimal256,
    borrow_ltv_aim: Decimal256,
) -> BorrowerActionResponse {
    //repay loan if you can't manage it
    if basset_price == Decimal256::zero() || locked_basset_amount == Uint256::zero() {
        if borrowed_amount > Uint256::zero() {
            return BorrowerActionResponse::repay(borrowed_amount);
        } else {
            return BorrowerActionResponse::nothing();
        }
    }

    let max_borrow_amount: Uint256 = locked_basset_amount * basset_price * basset_max_ltv;
    let current_ltv: Decimal256 =
        Decimal256::from_uint256(borrowed_amount) / Decimal256::from_uint256(max_borrow_amount);

    if current_ltv >= borrow_ltv_max {
        let aim_borrow_amount = borrow_ltv_aim * max_borrow_amount;
        let repay_amount = borrowed_amount - aim_borrow_amount;
        //TODO: buffer_size should be (max_ltv - aim_ltv)+0.15, to be able to repay loan directly
        //from buffer with high probability
        BorrowerActionResponse::repay(repay_amount)
    } else if current_ltv <= borrow_ltv_min {
        let aim_borrow_amount = borrow_ltv_aim * max_borrow_amount;
        let borrow_amount = aim_borrow_amount - borrowed_amount;
        //TODO: buffer_size should be (max_ltv - aim_ltv)+0.15, to be able to repay loan directly
        //from buffer with high probability
        BorrowerActionResponse::borrow(borrow_amount)
    } else {
        BorrowerActionResponse::nothing()
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_bignumber::{Decimal256, Uint256};
    use std::str::FromStr;
    use yield_optimizer::basset_farmer_config::BorrowerActionResponse;

    use super::calc_borrower_action;

    #[test]
    fn repay_loan() {
        let basset_price = Decimal256::from_str("5.5").unwrap();
        let borrowed_amount = Uint256::from(519_750u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 519_750 / 577_500 = 0.9
        //to_repay =  (0.9 - 0.8) * 577_500 = 57_750
        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(Uint256::from(57_750u64))
        );
    }

    #[test]
    fn borrow_more() {
        let basset_price = Decimal256::from_str("5.5").unwrap();
        let borrowed_amount = Uint256::from(346_500u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 346_500 / 577_500 = 0.6
        //to_borrow =  (0.8 - 0.6) * 577_500 = 115_500
        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::borrow(Uint256::from(115_500u64))
        );
    }

    #[test]
    fn do_nothing() {
        let basset_price = Decimal256::from_str("5.5").unwrap();
        let borrowed_amount = Uint256::from(473_550u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 473_550 / 577_500 = 0.82
        //0.75 < 0.82 < 0.85 => do nothing
        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }

    #[test]
    fn locked_amount_is_zero_and_borrowed_zero() {
        let basset_price = Decimal256::from_str("5.5").unwrap();
        let borrowed_amount = Uint256::zero();
        let locked_basset_amount = Uint256::zero();
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(borrower_action, BorrowerActionResponse::nothing());
    }

    #[test]
    fn locked_amount_is_zero_and_borrowed_not_zero() {
        let basset_price = Decimal256::from_str("5.5").unwrap();
        let borrowed_amount = Uint256::from(473_550u64);
        let locked_basset_amount = Uint256::zero();
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(borrowed_amount)
        );
    }

    #[test]
    fn borrowed_amount_is_zero() {
        let basset_price = Decimal256::from_str("5.5").unwrap();
        let borrowed_amount = Uint256::zero();
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        //max_borrow = 210_000 * 5.5 * 0.5 = 577_500
        //ltv = 0.0
        //to_borrow =  (0.8 - 0.0) * 577_500 = 462_000
        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::borrow(Uint256::from(462_000u64))
        );
    }

    #[test]
    fn asset_price_is_zero() {
        let basset_price = Decimal256::zero();
        let borrowed_amount = Uint256::from(473_550u64);
        let locked_basset_amount = Uint256::from(210_000u64);
        let basset_max_ltv = Decimal256::from_str("0.5").unwrap();
        let borrow_ltv_max = Decimal256::from_str("0.85").unwrap();
        let borrow_ltv_min = Decimal256::from_str("0.75").unwrap();
        let borrow_ltv_aim = Decimal256::from_str("0.8").unwrap();

        let borrower_action = calc_borrower_action(
            basset_price,
            borrowed_amount,
            locked_basset_amount,
            basset_max_ltv,
            borrow_ltv_max,
            borrow_ltv_min,
            borrow_ltv_aim,
        );
        assert_eq!(
            borrower_action,
            BorrowerActionResponse::repay(borrowed_amount)
        );
    }
}
