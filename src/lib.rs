use wasm_minimal_protocol::*;
use regex::bytes::Regex;
use once_cell::sync::Lazy;

initiate_protocol!();

enum MapType {
  Skip,
  Normal,
  NoSpace,
  SqrtN,
  MatrixBegin,
  MatrixEnd,
  Command,
  Default,
}

// a global static map list, from regex to string
static MAP_LIST: Lazy<Vec<(Regex, &[u8], MapType)>> = Lazy::new(|| { vec![
  (Regex::new(r"^ +").unwrap(), b"", MapType::Skip),
  (Regex::new(r"^,").unwrap(), b"\\,", MapType::Normal),
  (Regex::new(r"^;").unwrap(), b"\\;", MapType::Normal),
  (Regex::new(r"^/").unwrap(), b"\\/", MapType::Normal),
  (Regex::new(r"^\{").unwrap(), b"(", MapType::NoSpace),
  (Regex::new(r"^\} *\{").unwrap(), b", ", MapType::NoSpace),
  (Regex::new(r"^\}").unwrap(), b")", MapType::NoSpace),
  (Regex::new(r"^\(").unwrap(), b"(", MapType::NoSpace),
  (Regex::new(r"^\)").unwrap(), b")", MapType::NoSpace),
  (Regex::new(r"^\[").unwrap(), b"[", MapType::NoSpace),
  (Regex::new(r"^\]").unwrap(), b"]", MapType::NoSpace),
  (Regex::new(r"^\\\{").unwrap(), b"{", MapType::NoSpace),
  (Regex::new(r"^\\\}").unwrap(), b"}", MapType::NoSpace),
  (Regex::new(r"^\^").unwrap(), b"^", MapType::NoSpace),
  (Regex::new(r"^\_").unwrap(), b"_", MapType::NoSpace),
  (Regex::new(r"^\\frac *\{").unwrap(), b"frac(", MapType::Normal),
  (Regex::new(r"^\ qrt *\[ *([0-9]+) *\] *\{").unwrap(), b"", MapType::SqrtN),
  (Regex::new(r"^\ qrt *\{").unwrap(), b"sqrt(", MapType::Normal),
  (Regex::new(r"^\\begin\{matrix\}").unwrap(), b"matrix(delim: none,", MapType::MatrixBegin),
  (Regex::new(r"^\\begin\{pmatrix\}").unwrap(), b"matrix(delim: \"(\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin\{bmatrix\}").unwrap(), b"matrix(delim: \"[\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin\{Bmatrix\}").unwrap(), b"matrix(delim: \"{\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin\{vmatrix\}").unwrap(), b"matrix(delim: \"|\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin\{Vmatrix\}").unwrap(), b"matrix(delim: \"||\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin\{array\}\{[lcr\|]+\}").unwrap(), b"matrix(delim: none,", MapType::MatrixBegin),
  (Regex::new(r"^\\end\{[pbBvV]?matrix\}").unwrap(), b")", MapType::MatrixEnd),
  (Regex::new(r"^\\end\{array\}").unwrap(), b")", MapType::MatrixEnd),
  (Regex::new(r"^\\,").unwrap(), b"thin", MapType::Normal),
  (Regex::new(r"^\\;").unwrap(), b"med", MapType::Normal),
  (Regex::new(r"^~").unwrap(), b"thick", MapType::Normal),
  (Regex::new(r"^\\ ").unwrap(), b"thick", MapType::Normal),
  (Regex::new(r"^\\([a-zA-Z]+)").unwrap(), b"", MapType::Command),
  (Regex::new(r"^[a-zA-Z+\-*!<>=]").unwrap(), b"", MapType::Default),
  (Regex::new(r"^[0-9]+").unwrap(), b"", MapType::Default),
]});


#[wasm_func]
pub fn convert(input: &[u8]) -> Result<Vec<u8>, String> {
  // mutable Vec<u8> to store the output
  let mut output: Vec<u8> = Vec::new();
  // loop to eat the input
  let mut i = 0;
  while i < input.len() {
    // find the first match
    let mut matched = false;
    for (regex, replacement, map_type) in MAP_LIST.iter() {
      if let Some(m) = regex.captures(&input[i..]) {
        // map the matched string
        match map_type {
          MapType::Skip => {},
          MapType::Normal => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
          },
          MapType::NoSpace => {
            output.extend_from_slice(replacement);
          },
          MapType::SqrtN => {
            output.extend_from_slice(b"root(");
            // group 1
            output.extend_from_slice(&input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()]);
            output.extend_from_slice(b", ");
          },
          MapType::MatrixBegin => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
          },
          MapType::MatrixEnd => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
          },
          MapType::Command => {
            // group 1
            output.extend_from_slice(&input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()]);
            output.extend_from_slice(b" ");
          },
          MapType::Default => {
            // group 0
            output.extend_from_slice(&input[i..i+m.get(0).unwrap().end()]);
            output.extend_from_slice(b" ");
          },
        }
        // move the index
        i += m.get(0).unwrap().end();
        matched = true;
        break;
      }
    }
    if !matched {
      // if not matched, panic
      return Result::Err(String::from(format!("not matched for \"{}\"", String::from_utf8_lossy(&input[i..]))));
    }
  }
  Result::Ok(output)
}


// test with b"abc"
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert() {
    assert_eq!(convert(b"ab?c").unwrap(), b"a b c ");
  }
}
