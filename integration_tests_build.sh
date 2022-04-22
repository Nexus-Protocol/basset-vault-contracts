# Set `CONTRACTS_SCRIPTS_PATH` environement variable.
# It should points to the contracts scripts directory on your machine

# You can add more contracts that are built with integration tests to this array
contracts=(
    "basset_vault"
    "basset_vault_strategy"
)

for contract in "${contracts[@]}"
do
    cd "contracts/${contract}"
    cargo build --release --features integration_tests_build --target=wasm32-unknown-unknown
    cd ../..
    wasm-strip "target/wasm32-unknown-unknown/release/basset_vault_${contract}.wasm"
    cp "target/wasm32-unknown-unknown/release/basset_vault_${contract}.wasm" "${CONTRACTS_SCRIPTS_PATH}/wasm_artifacts/nexus/basset_vaults_integration_tests/basset_vault_${contract}.wasm"
done
