//! This is a WASM wrapper of the MiTeX library for Web browsers.

#![allow(missing_docs)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Converts the A LaTex math equation into a plain text
///
/// # Errors
/// Returns an error if the input is not a valid utf-8 string
/// Returns an error if the input is not a valid math equation
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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
