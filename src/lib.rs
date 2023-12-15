use wasm_minimal_protocol::*;
use regex::bytes::Regex;
use once_cell::sync::Lazy;

initiate_protocol!();

enum MapType {
  Skip,  // skip space and newline
  Normal,  // normal map, add a space after the replacement
  NoSpace,  // normal map, but no space after the replacement
  SqrtN,  // sqrt with n
  MatrixBegin,  // matrix begin, for convert & to , and \\ to ;
  MatrixEnd,  // matrix end, for convert & to & and \\ to \
  Command,  // commands like "\alpha", add a space after the command
  Default,  // straight copy, add a space after the string
}

// a global static map list, from regex to string
static TOKEN_MAP_LIST: Lazy<Vec<(Regex, &[u8], MapType)>> = Lazy::new(|| { vec![
  // Spaces and newline
  (Regex::new(r"^[ \n]+").unwrap(), b"", MapType::Skip),
  // Escape symbols
  (Regex::new(r"^,").unwrap(), b"\\,", MapType::Normal),
  (Regex::new(r"^;").unwrap(), b"\\;", MapType::Normal),
  (Regex::new(r"^/").unwrap(), b"\\/", MapType::Normal),
  // Just a hack for "}{" in "frac{}{}"
  (Regex::new(r"^\{").unwrap(), b"(", MapType::NoSpace),
  (Regex::new(r"^\}[ \n]*\{").unwrap(), b", ", MapType::NoSpace),
  (Regex::new(r"^\}").unwrap(), b")", MapType::NoSpace),
  // Left/Right with .
  (Regex::new(r"^\\left[ \n]*\.").unwrap(), b"lr(", MapType::Normal),
  (Regex::new(r"^\\right[ \n]*\.").unwrap(), b")", MapType::Normal),
  // Brackets
  (Regex::new(r"^\(").unwrap(), b"(", MapType::NoSpace),
  (Regex::new(r"^\)").unwrap(), b")", MapType::NoSpace),
  (Regex::new(r"^\[").unwrap(), b"[", MapType::NoSpace),
  (Regex::new(r"^\]").unwrap(), b"]", MapType::NoSpace),
  (Regex::new(r"^\\\{").unwrap(), b"{", MapType::NoSpace),
  (Regex::new(r"^\\\}").unwrap(), b"}", MapType::NoSpace),
  // Sups and subs
  (Regex::new(r"^\^").unwrap(), b"^", MapType::NoSpace),
  (Regex::new(r"^\_").unwrap(), b"_", MapType::NoSpace),
  // Frac
  (Regex::new(r"^\\([tdc])?frac[ \n]*\{").unwrap(), b"frac(", MapType::Normal),
  // Sqrt
  (Regex::new(r"^\\sqrt[ \n]*\[[ \n]*([0-9]+)[ \n]*\][ \n]*\{").unwrap(), b"", MapType::SqrtN),
  (Regex::new(r"^\\sqrt[ \n]*\{").unwrap(), b"sqrt(", MapType::Normal),
  // Aligned
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*aligned[ \n]*\}").unwrap(), b"", MapType::Normal),
  (Regex::new(r"^\\end[ \n]*\{[ \n]*aligned[ \n]*\}").unwrap(), b"", MapType::Normal),
  // Matrices
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*matrix[ \n]*\}").unwrap(), b"mat(delim: #none,", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*pmatrix[ \n]*\}").unwrap(), b"mat(delim: \"(\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*bmatrix[ \n]*\}").unwrap(), b"mat(delim: \"[\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*Bmatrix[ \n]*\}").unwrap(), b"mat(delim: \"{\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*vmatrix[ \n]*\}").unwrap(), b"mat(delim: \"|\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*Vmatrix[ \n]*\}").unwrap(), b"mat(delim: \"||\",", MapType::MatrixBegin),
  (Regex::new(r"^\\begin[ \n]*\{[ \n]*array[ \n]*\}\{[lcr\| \n]+\}").unwrap(), b"mat(delim: #none,", MapType::MatrixBegin),
  (Regex::new(r"^\\end[ \n]*\{[ \n]*[pbBvV]?matrix[ \n]*\}").unwrap(), b")", MapType::MatrixEnd),
  (Regex::new(r"^\\end[ \n]*\{[ \n]*array[ \n]*\}").unwrap(), b")", MapType::MatrixEnd),
  // Spaces
  (Regex::new(r"^\\!").unwrap(), b"#h(-(1/6)*1em);", MapType::Normal),
  (Regex::new(r"^\\,").unwrap(), b"thin", MapType::Normal),
  (Regex::new(r"^\\>").unwrap(), b"med", MapType::Normal),
  (Regex::new(r"^\\:").unwrap(), b"med", MapType::Normal),
  (Regex::new(r"^\\;").unwrap(), b"thick", MapType::Normal),
  (Regex::new(r"^\\[ \n]").unwrap(), b"thick", MapType::Normal),
  (Regex::new(r"^~").unwrap(), b"thick", MapType::Normal),
  // Commands and default
  (Regex::new(r"^\\([a-zA-Z]+)").unwrap(), b"", MapType::Command),
  (Regex::new(r"^[a-zA-Z+\-*!<>=]").unwrap(), b"", MapType::Default),
  (Regex::new(r"^[0-9]+").unwrap(), b"", MapType::Default),
]});


#[wasm_func]
pub fn convert(input: &[u8]) -> Result<Vec<u8>, String> {
  // mutable Vec<u8> to store the output
  let mut output: Vec<u8> = Vec::new();
  // count for detect whether the mode is matrix
  let mut matrix_count = 0;
  // loop to eat the input
  let mut i = 0;
  while i < input.len() {
    // special handle for & and \\
    if input[i] == b'&' {
      if matrix_count > 0 {
        output.extend_from_slice(b", ");
      } else {
        output.extend_from_slice(b"& ");
      }
      i += 1;
      continue;
    } else if input[i] == b'\\' && i+1 < input.len() && input[i+1] == b'\\' {
      if matrix_count > 0 {
        output.extend_from_slice(b"; ");
      } else {
        output.extend_from_slice(b"\\ ");
      }
      i += 2;
      continue;
    }
    // find the first match
    let mut matched = false;
    for (regex, replacement, map_type) in TOKEN_MAP_LIST.iter() {
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
            matrix_count += 1;
          },
          MapType::MatrixEnd => {
            output.extend_from_slice(replacement);
            output.extend_from_slice(b" ");
            matrix_count -= 1;
            if matrix_count < 0 {
              return Result::Err(String::from(format!("matrix environment end without begin at {}", i)));
            }
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
      return Result::Err(String::from(format!("no matched pattern for \"{}\"", String::from_utf8_lossy(&input[i..]))));
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
    assert_eq!(convert(b"abc").unwrap(), b"a b c ");
  }
}
