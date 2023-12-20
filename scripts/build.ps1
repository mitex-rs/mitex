cargo run --release --bin mitex-cli
cargo build --release --target wasm32-unknown-unknown -p mitex-typst
rm packages/mitex/mitex.wasm -Force
mv target/wasm32-unknown-unknown/release/mitex_typst.wasm packages/mitex/mitex.wasm