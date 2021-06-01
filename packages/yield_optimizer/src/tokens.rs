//TODO: remove me
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, Api, CanonicalAddr, Deps, Querier, StdError, StdResult, Storage};

pub type TokenCanonical = (CanonicalAddr, Uint256);
pub type TokenAddr = (Addr, Uint256);
pub type TokenString = (String, Uint256);

pub type TokensCanonical = Vec<TokenCanonical>;
pub type TokensAddr = Vec<TokenAddr>;
pub type TokensString = Vec<TokenString>;

pub trait TokensMath {
    fn sub(&mut self, collaterals: TokensCanonical) -> StdResult<()>;
    fn add(&mut self, collaterals: TokensCanonical);
}

pub trait TokensValidate {
    fn validate(&self, api: &dyn Api) -> StdResult<TokensAddr>;
}

pub trait TokensToAddr {
    fn to_human(&self, api: &dyn Api) -> StdResult<TokensAddr>;
}

pub trait TokensToCanonical {
    fn to_raw(&self, api: &dyn Api) -> StdResult<TokensCanonical>;
}

impl TokensMath for TokensCanonical {
    fn sub(&mut self, tokens: TokensCanonical) -> StdResult<()> {
        self.sort_by(|a, b| {
            let res = a.0.as_slice().cmp(&b.0.as_slice());
            if res == std::cmp::Ordering::Equal {
                panic!("Invalid Tokens")
            }

            res
        });

        let mut tokens = tokens;
        tokens.sort_by(|a, b| {
            let res = a.0.as_slice().cmp(&b.0.as_slice());
            if res == std::cmp::Ordering::Equal {
                panic!("Invalid Tokens")
            }

            res
        });

        let mut i = 0;
        let mut j = 0;
        while i < self.len() && j < tokens.len() {
            if self[i].0 == tokens[j].0 {
                if self[i].1 < tokens[j].1 {
                    return Err(StdError::generic_err("Subtraction underflow"));
                }

                self[i].1 = self[i].1 - tokens[j].1;

                i += 1;
                j += 1;
            } else if self[i].0.as_slice().cmp(&tokens[j].0.as_slice()) == std::cmp::Ordering::Less
            {
                i += 1;
            } else {
                return Err(StdError::generic_err("Subtraction underflow"));
            }
        }

        if j != tokens.len() {
            return Err(StdError::generic_err("Subtraction underflow"));
        }

        // remove zero tokens
        self.retain(|v| v.1 > Uint256::zero());

        Ok(())
    }

    fn add(&mut self, tokens: TokensCanonical) {
        self.sort_by(|a, b| {
            let res = a.0.as_slice().cmp(&b.0.as_slice());
            if res == std::cmp::Ordering::Equal {
                panic!("Invalid Tokens")
            }

            res
        });

        let mut tokens = tokens;
        tokens.sort_by(|a, b| {
            let res = a.0.as_slice().cmp(&b.0.as_slice());
            if res == std::cmp::Ordering::Equal {
                panic!("Invalid Tokens")
            }

            res
        });

        let mut i = 0;
        let mut j = 0;
        while i < self.len() && j < tokens.len() {
            if self[i].0 == tokens[j].0 {
                self[i].1 += tokens[j].1;

                i += 1;
                j += 1;
            } else if self[i].0.as_slice().cmp(&tokens[j].0.as_slice())
                == std::cmp::Ordering::Greater
            {
                j += 1;
            } else {
                i += 1;
            }
        }

        while j < tokens.len() {
            self.push(tokens[j].clone());
            j += 1;
        }

        // remove zero tokens
        self.retain(|v| v.1 > Uint256::zero());
    }
}

impl TokensToAddr for TokensCanonical {
    fn to_human(&self, api: &dyn Api) -> StdResult<TokensAddr> {
        let collaterals: TokensAddr = self
            .iter()
            .map(|c| Ok((api.addr_humanize(&c.0)?, c.1)))
            .collect::<StdResult<TokensAddr>>()?;
        Ok(collaterals)
    }
}

impl TokensToCanonical for TokensAddr {
    fn to_raw(&self, api: &dyn Api) -> StdResult<TokensCanonical> {
        let tokens: TokensCanonical = self
            .iter()
            .map(|c| Ok((api.addr_canonicalize(&c.0.to_string())?, c.1)))
            .collect::<StdResult<TokensCanonical>>()?;
        Ok(tokens)
    }
}

impl TokensValidate for TokensString {
    fn validate(&self, api: &dyn Api) -> StdResult<TokensAddr> {
        let validated: TokensAddr = self
            .iter()
            .map(|c| Ok((api.addr_validate(&c.0)?, c.1)))
            .collect::<StdResult<TokensAddr>>()?;
        Ok(validated)
    }
}
