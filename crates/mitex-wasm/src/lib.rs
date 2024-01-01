//! This is a WASM wrapper of the MiTeX library for Typst.
//!
//! # Usage
//!
//! For example, if you want to call [`convert_math`] function in Typst, you can
//! write the following code in your Typst file:
//!
//! ```typ
//! #let mitex-wasm = plugin("./mitex.wasm")
//!
//! #let mitex-convert(it: "", spec: bytes(())) = {
//!   str(mitex-wasm.convert_math(bytes(it), spec))
//! }
//! ```

// todo: maybe a bug of wasm_minimal_protocol.
// #[cfg_attr(target_arch = "wasm32", wasm_func)]
// |                                  ^^^^^^^^^
#![allow(missing_docs)]

mod impls {
    #[cfg(feature = "web")]
    pub use wasm_bindgen::prelude::*;

    /// Converts a json command specification into a binary command
    /// specification
    ///
    /// # Errors
    /// Returns an error if the input is not a valid json string
    #[cfg(feature = "spec-api")]
    #[cfg_attr(feature = "web", wasm_bindgen)]
    pub fn compile_spec(input: &[u8]) -> Result<Vec<u8>, String> {
        let res: mitex_spec::JsonCommandSpec =
            serde_json::from_slice(input).map_err(|e| e.to_string())?;
        let res: mitex_spec::CommandSpec = res.into();
        Result::Ok(res.to_bytes())
    }

    /// Converts a LaTeX math equation into a plain text
    ///
    /// # Errors
    /// Returns an error if the input is not a valid math equation
    #[cfg_attr(feature = "web", wasm_bindgen)]
    pub fn convert_math(input: &str, spec: &[u8]) -> Result<String, String> {
        let spec = if spec.is_empty() {
            None
        } else {
            let spec = mitex_spec::CommandSpec::from_bytes(spec);
            Some(spec)
        };
        let res = mitex::convert_math(input, spec)?;
        Result::Ok(res)
    }

    /// Converts a LaTeX code into a plain text
    ///
    /// # Errors
    /// Returns an error if the input is not a valid LaTeX code
    #[cfg_attr(feature = "web", wasm_bindgen)]
    pub fn convert_text(input: &str, spec: &[u8]) -> Result<String, String> {
        let spec = if spec.is_empty() {
            None
        } else {
            let spec = mitex_spec::CommandSpec::from_bytes(spec);
            Some(spec)
        };
        let res = mitex::convert_text(input, spec)?;
        Result::Ok(res)
    }
}

/// Wrappers for Typst as the host
#[cfg(feature = "typst-plugin")]
mod wasm_host {
    pub use wasm_minimal_protocol::*;
    initiate_protocol!();

    fn wasm_into_str(input: &[u8]) -> Result<&str, String> {
        std::str::from_utf8(input).map_err(|e| e.to_string())
    }

    #[cfg(feature = "spec-api")]
    #[cfg_attr(feature = "typst-plugin", wasm_func)]
    pub fn compile_spec(input: &[u8]) -> Result<Vec<u8>, String> {
        super::impls::compile_spec(input)
    }

    /// See [`super::impls::convert_math`]
    ///
    /// # Errors
    /// Returns an error if the input is not a valid utf-8 string
    #[cfg_attr(feature = "typst-plugin", wasm_func)]
    pub fn convert_math(input: &[u8], spec: &[u8]) -> Result<Vec<u8>, String> {
        let input = wasm_into_str(input)?;
        let res = super::impls::convert_math(input, spec)?;
        Result::Ok(res.into_bytes())
    }

    /// See [`super::impls::convert_text`]
    ///
    /// # Errors
    /// Returns an error if the input is not a valid utf-8 string
    #[cfg_attr(feature = "typst-plugin", wasm_func)]
    pub fn convert_text(input: &[u8], spec: &[u8]) -> Result<Vec<u8>, String> {
        let input = wasm_into_str(input)?;
        let res = super::impls::convert_text(input, spec)?;
        Result::Ok(res.into_bytes())
    }
}

/// Wrappers for Browsers as the host
#[cfg(not(feature = "typst-plugin"))]
mod wasm_host {
    pub use super::impls::*;
}

pub use wasm_host::*;

// test with b"abc"
#[cfg(test)]
#[cfg(feature = "typst-plugin")]
mod tests {
    use super::*;

    #[test]
    fn test_convert_math() {
        assert_eq!(convert_math(b"$abc$", &[]).unwrap(), b"a b c ");
    }

    #[test]
    fn test_convert_text() {
        assert_eq!(convert_text(b"abc", &[]).unwrap(), b"abc");
    }
}
