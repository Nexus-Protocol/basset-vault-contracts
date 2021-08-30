pub mod anchor;
pub mod basset_vault;
pub mod basset_vault_strategy;
pub mod common;
pub mod nasset_token;
pub mod nasset_token_config_holder;
pub mod nasset_token_rewards;
pub mod psi_distributor;
pub mod querier;
pub mod terraswap;
pub mod terraswap_pair;

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
