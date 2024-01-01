cargo build --release --target wasm32-unknown-unknown --manifest-path ./crates/mitex-wasm/Cargo.toml --features typst-plugin
$InstallPath = "packages/mitex/mitex.wasm"
if (Test-Path $InstallPath) {
  Remove-Item $InstallPath
}
mv target/wasm32-unknown-unknown/release/mitex_typst.wasm $InstallPath

pwsh -Command { cd crates/mitex-wasm; wasm-pack build --release --features web }
