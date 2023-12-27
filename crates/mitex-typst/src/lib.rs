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

// test with b"abc"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        assert_eq!(convert_math(b"$abc$", &[]).unwrap(), b"a b c ");
    }
}
