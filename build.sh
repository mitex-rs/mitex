cargo build --release --target wasm32-unknown-unknown
mv target/wasm32-unknown-unknown/release/mitex.wasm typst-package/mitex.wasm