# Set `CONTRACTS_SCRIPTS_PATH` environement variable.
# It should points to the contracts scripts directory on your machine

# Basset vault
cd contracts/basset_vault
cargo build --release --features integration_tests_build --target=wasm32-unknown-unknown
cd ../..
wasm-strip target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm
cp target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm "${CONTRACTS_SCRIPTS_PATH}/wasm_artifacts/nexus/basset_vaults_integration_tests/basset_vault_basset_vault.wasm"

# You can add more contracts that are built with integration tests flag using the template above