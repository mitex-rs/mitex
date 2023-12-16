extern crate mitex;
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn convert_math(input: &[u8]) -> Result<Vec<u8>, String> {
    let input = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let res = mitex::convert_math(input)?;
    Result::Ok(res.into_bytes())
}

// test with b"abc"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        assert_eq!(convert_math(b"$abc$").unwrap(), b"a b c ");
    }
}
