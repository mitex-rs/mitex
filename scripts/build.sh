cargo build --release --target wasm32-unknown-unknown -p mitex-typst
rm -f packages/mitex/mitex.wasm
mv target/wasm32-unknown-unknown/release/mitex_typst.wasm packages/mitex/mitex.wasm