cargo build --release --target wasm32-unknown-unknown -p mitex-typst
rm -f typst-package/mitex.wasm
mv target/wasm32-unknown-unknown/release/mitex_typst.wasm typst-package/mitex.wasm