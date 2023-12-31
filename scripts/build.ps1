cargo run --release --bin mitex
cargo build --release --target wasm32-unknown-unknown -p mitex-typst
cp target/wasm32-unknown-unknown/release/mitex_typst.wasm packages/mitex/mitex.wasm

cd crates/mitex-web-wasm
wasm-pack build --release
cd ../..
