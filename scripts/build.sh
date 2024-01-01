#!/usr/bin/env bash
set -euxo pipefail
cargo build --release --target wasm32-unknown-unknown --manifest-path ./crates/mitex-wasm/Cargo.toml --features typst-plugin
cp target/wasm32-unknown-unknown/release/mitex_wasm.wasm packages/mitex/mitex.wasm

pushd crates/mitex-wasm
wasm-pack build --release --features web
popd
