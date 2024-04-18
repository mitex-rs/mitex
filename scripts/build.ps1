cargo build --release --target wasm32-unknown-unknown --manifest-path ./crates/mitex-wasm/Cargo.toml --features typst-plugin
$InstallPath = "packages/mitex/mitex.wasm"
if (Test-Path $InstallPath) {
  Remove-Item $InstallPath
}
Move-Item target/wasm32-unknown-unknown/release/mitex_wasm.wasm $InstallPath

pwsh -Command { Set-Location crates/mitex-wasm; wasm-pack build --release --features web }
