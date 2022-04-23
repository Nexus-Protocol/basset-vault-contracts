# We use workspace-optimizer container as default and recommended way
# to build optimized smart contracts binaries 
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.5

# But bvault require manual build because of its size.
# We use nightly flags that strip debug messages in panicking in rusts standart library
cd contracts/basset_vault
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target wasm32-unknown-unknown --release
cd ../..
wasm-strip target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm
wasm-opt -Oz target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm -o target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm
cp target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm artifacts/basset_vault_basset_vault.wasm

# Regenerate checksum for `basset_vault` because of out custom build

# Intermediate checksum
grep -v "target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm" artifacts/checksums_intermediate.txt > checksums_intermediate.txt.tmp && mv checksums_intermediate.txt.tmp artifacts/checksums_intermediate.txt
sha256sum -- "target/wasm32-unknown-unknown/release/basset_vault_basset_vault.wasm" | tee -a artifacts/checksums_intermediate.txt

# Checksum
grep -v "basset_vault_basset_vault.wasm" artifacts/checksums.txt > artifacts/checksums.txt.tmp && mv artifacts/checksums.txt.tmp artifacts/checksums.txt
cd artifacts
sha256sum -- "basset_vault_basset_vault.wasm" | tee -a checksums.txt
