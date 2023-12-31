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

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::*;

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[cfg(feature = "spec-api")]
#[wasm_func]
pub fn compile_spec(input: &[u8]) -> Result<Vec<u8>, String> {
    let res: mitex_spec::JsonCommandSpec =
        serde_json::from_slice(input).map_err(|e| e.to_string())?;
    let res: mitex_spec::CommandSpec = res.into();
    Result::Ok(res.to_bytes())
}

/// Converts the A LaTeX math equation into a plain text
///
/// # Errors
/// Returns an error if the input is not a valid utf-8 string
/// Returns an error if the input is not a valid math equation
#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn convert_math(input: &[u8], spec: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let spec = if spec.is_empty() {
        None
    } else {
        let spec = mitex_spec::CommandSpec::from_bytes(spec);
        Some(spec)
    };
    let res = mitex::convert_math(input, spec)?;
    Result::Ok(res.into_bytes())
}

/// Converts the A LaTeX string into a plain text
///
/// # Errors
/// Returns an error if the input is not a valid utf-8 string
/// Returns an error if the input is not a valid LaTeX string
#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn convert_text(input: &[u8], spec: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let spec = if spec.is_empty() {
        None
    } else {
        let spec = mitex_spec::CommandSpec::from_bytes(spec);
        Some(spec)
    };
    let res = mitex::convert_text(input, spec)?;
    Result::Ok(res.into_bytes())
}

// test with b"abc"
#[cfg(test)]
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
