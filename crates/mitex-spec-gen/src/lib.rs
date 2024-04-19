//! Provides embedded command specifications for MiTeX.

use mitex_spec::CommandSpec;

/// The default command specification.
///
/// See [Reproducing Default Command Specification][repro-default] for more
/// information.
///
/// [repro-default]: https://github.com/mitex-rs/artifacts/blob/main/README.md#default-command-specification-since-v011
pub static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(|| {
    CommandSpec::from_bytes(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/mitex-artifacts/spec/default.rkyv"
    )))
});
