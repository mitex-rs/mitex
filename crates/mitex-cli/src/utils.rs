//! Utility functions and types.

/// A wrapper around `Box<str>` that implements `std::error::Error`.
pub struct Error(Box<str>);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self(s.into_boxed_str())
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self(s.to_owned().into_boxed_str())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self(err.to_string().into_boxed_str())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self(err.to_string().into_boxed_str())
    }
}

/// Exit with an error message.
pub fn exit_with_error<E: std::error::Error>(err: E) -> ! {
    clap::Error::raw(
        clap::error::ErrorKind::ValueValidation,
        format!("mitex error: {err}"),
    )
    .exit()
}

/// Exit with an error message.
pub trait UnwrapOrExit<T> {
    /// Unwrap the result or exit with an error message.
    fn unwrap_or_exit(self) -> T;
}

impl<T, E: std::error::Error> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        self.map_err(exit_with_error).unwrap()
    }
}
