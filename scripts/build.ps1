cargo run --release --bin mitex
cargo build --release --target wasm32-unknown-unknown --manifest-path ./crates/mitex-wasm/Cargo.toml --features typst-plugin
cp target/wasm32-unknown-unknown/release/mitex_typst.wasm packages/mitex/mitex.wasm

pwsh -Command { cd crates/mitex-wasm; wasm-pack build --release --features web }
