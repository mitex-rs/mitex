# Contributing to MiTeX

Currently, MiTeX maintains following three parts of code:

- A TeX parser library written in **Rust**, see [mitex-lexer](https://github.com/mitex-rs/mitex/tree/main/crates/mitex-lexer) and [mitex-parser](https://github.com/mitex-rs/mitex/tree/main/crates/mitex-parser).
- A TeX to Typst converter library written in **Rust**, see [mitex](https://github.com/mitex-rs/mitex/tree/main/crates/mitex).
- A list of TeX packages and commands written in **Typst**, which then used by the typst package, see [MiTeX Command Specification](https://github.com/mitex-rs/mitex/tree/main/packages/mitex/specs).

For a translation process, for example, we have:

```
\frac{1}{2}

===[parser]===> AST ===[converter]===>

#eval("$frac(1, 2)$", scope: (frac: (num, den) => $(num)/(den)$))
```

You can use the `#mitex-convert()` function to get the Typst Code generated from LaTeX Code.

### Adding missing TeX commands

Even if you don't know Rust at all, you can still add missing TeX commands to MiTeX by modifying [specification files](https://github.com/mitex-rs/mitex/tree/main/packages/mitex/specs), since they are written in typst! You can open an issue to acquire the commands you want to add, or you can edit the files and submit a pull request.

In the future, we will provide the ability to customize TeX commands, which will make it easier for you to use the commands you create for yourself.

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
