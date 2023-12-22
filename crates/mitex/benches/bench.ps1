echo "/*"
hyperfine.exe 'typst compile --root . crates\mitex\benches\empty.typ' --warmup 3
hyperfine.exe 'typst compile --root . crates\mitex\benches\oiwiki.typ' --warmup 3
hyperfine.exe 'typst compile --root . crates\mitex\benches\oiwiki-with-render.typ' --warmup 3
cargo bench --manifest-path .\crates\mitex\Cargo.toml
echo "*/"