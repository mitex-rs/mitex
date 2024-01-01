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

## Documents

TODO.
