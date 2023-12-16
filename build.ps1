cargo build --release --target wasm32-unknown-unknown
rm typst-package/mitex.wasm
mv target/wasm32-unknown-unknown/release/mitex_typst.wasm typst-package/mitex.wasm