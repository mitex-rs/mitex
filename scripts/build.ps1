cargo build --release --target wasm32-unknown-unknown -p mitex-typst
$InstallPath = "packages/mitex/mitex.wasm"
if (Test-Path $InstallPath) {
  Remove-Item $InstallPath
}
mv target/wasm32-unknown-unknown/release/mitex_typst.wasm $InstallPath