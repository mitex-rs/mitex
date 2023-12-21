pub mod lexer {
    use mitex_spec::CommandSpec;

    pub static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> =
        once_cell::sync::Lazy::new(|| {
            CommandSpec::from_bytes(include_bytes!(
                "../../../../target/mitex-artifacts/spec/default.rkyv"
            ))
        });
}

pub use lexer::*;
