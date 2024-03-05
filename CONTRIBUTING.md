# Contributing to MiTeX

## Installing Dependencies

You should install [Typst](https://github.com/typst/typst?tab=readme-ov-file#installation) and [Rust](https://www.rust-lang.org/tools/install) for running the build script.

If you want to build the WASM plugin, you should also setup the wasm target by rustup:

```sh
rustup target add wasm32-unknown-unknown
```

## Build

For Linux:

```sh
git clone https://github.com/mitex-rs/mitex.git
scripts/build.sh
```

For Windows:

```sh
git clone https://github.com/mitex-rs/mitex.git
.\scripts\build.ps1
```

## Fuzzing (Testing)

The [afl.rs] only supports Linux.

Installing [afl.rs] on Linux:

```bash
cargo install cargo-afl
```

Building and fuzzing:

```bash
cargo afl build --bin fuzz-target-mitex
cargo afl fuzz -i local/seed -o local/fuzz-res ./target/debug/fuzz-target-mitex
```

To minimize test cases, using `afl-tmin`

```bash
cargo afl tmin -i crash.tex -o minimized.tex ./target/debug/fuzz-target-mitex
```

## Documents

TODO.

[afl.rs]: https://github.com/rust-fuzz/afl.rs
