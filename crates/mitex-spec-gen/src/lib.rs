//! Generate Default Command Specification for embedded LaTeX packages.

use mitex_spec::CommandSpec;

/// Default Command Specification.
pub static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(|| {
    CommandSpec::from_bytes(include_bytes!(
        "../../../target/mitex-artifacts/spec/default.rkyv"
    ))
});
