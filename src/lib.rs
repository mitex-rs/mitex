use phf::phf_map;
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

// Command maps, [key] to (replacement, add_space)
static COMMAND_MAP: phf::Map<&'static [u8], (&'static [u8], bool)> = phf_map! {
  // greek letters
  b"alpha" => (b"alpha", true),
  b"beta" => (b"beta", true),
  b"gamma" => (b"gamma", true),
  b"delta" => (b"delta", true),
  b"epsilon" => (b"epsilon", true),
  b"zeta" => (b"zeta", true),
  b"eta" => (b"eta", true),
  b"theta" => (b"theta", true),
  b"iota" => (b"iota", true),
  b"kappa" => (b"kappa", true),
  b"lambda" => (b"lambda", true),
  b"mu" => (b"mu", true),
  b"nu" => (b"nu", true),
  b"xi" => (b"xi", true),
  b"omicron" => (b"omicron", true),
  b"pi" => (b"pi", true),
  b"rho" => (b"rho", true),
  b"sigma" => (b"sigma", true),
  b"tau" => (b"tau", true),
  b"upsilon" => (b"upsilon", true),
  b"phi" => (b"phi", true),
  b"chi" => (b"chi", true),
  b"psi" => (b"psi", true),
  b"omega" => (b"omega", true),
  b"Alpha" => (b"Alpha", true),
  b"Beta" => (b"Beta", true),
  b"Gamma" => (b"Gamma", true),
  b"Delta" => (b"Delta", true),
  b"Epsilon" => (b"Epsilon", true),
  b"Zeta" => (b"Zeta", true),
  b"Eta" => (b"Eta", true),
  b"Theta" => (b"Theta", true),
  b"Iota" => (b"Iota", true),
  b"Kappa" => (b"Kappa", true),
  b"Lambda" => (b"Lambda", true),
  b"Mu" => (b"Mu", true),
  b"Nu" => (b"Nu", true),
  b"Xi" => (b"Xi", true),
  b"Omicron" => (b"Omicron", true),
  b"Pi" => (b"Pi", true),
  b"Rho" => (b"Rho", true),
  b"Sigma" => (b"Sigma", true),
  b"Tau" => (b"Tau", true),
  b"Upsilon" => (b"Upsilon", true),
  b"Phi" => (b"Phi", true),
  b"Chi" => (b"Chi", true),
  b"Psi" => (b"Psi", true),
  b"Omega" => (b"Omega", true),
  // Symbols
  b"infty" => (b"oo", true),
  // Functions
  b"sin" => (b"sin", true),
  b"cos" => (b"cos", true),
  b"tan" => (b"tan", true),
  b"cot" => (b"cot", true),
  b"sec" => (b"sec", true),
  b"csc" => (b"csc", true),
  b"arcsin" => (b"arcsin", true),
  b"arccos" => (b"arccos", true),
  b"arctan" => (b"arctan", true),
  b"sinh" => (b"sinh", true),
  b"cosh" => (b"cosh", true),
  b"tanh" => (b"tanh", true),
  b"coth" => (b"coth", true),
  b"ln" => (b"ln", true),
  b"log" => (b"log", true),
  b"lg" => (b"lg", true),
  b"lim" => (b"lim", true),
  b"limsup" => (b"limsup", true),
  b"liminf" => (b"liminf", true),
  b"max" => (b"max", true),
  b"min" => (b"min", true),
  b"sup" => (b"sup", true),
  b"inf" => (b"inf", true),
  b"det" => (b"det", true),
  b"dim" => (b"dim", true),
  b"ker" => (b"ker", true),
  b"hom" => (b"hom", true),
  b"exp" => (b"exp", true),
  b"Pr" => (b"Pr", true),
  b"arg" => (b"arg", true),
  b"deg" => (b"deg", true),
  b"gcd" => (b"gcd", true),
  b"lcm" => (b"lcm", true),
  b"sum" => (b"sum", true),
  b"prod" => (b"product", true),
  // Integrals
  b"int" => (b"integral", true),
  b"iint" => (b"integral.double", true),
  b"iiint" => (b"integral.triple", true),
  b"oint" => (b"integral.cont", true),
  b"oiint" => (b"integral.surf", true),
  b"oiiint" => (b"integral.vol", true),
  // Operators
  b"mod" => (b"mod", true),
  b"cdot" => (b"dot.c", true),
  b"times" => (b"times", true),
  b"oplus" => (b"plus.circle", true),
  b"ominus" => (b"minus.circle", true),
  b"pm" => (b"plus.minus", true),
  b"mp" => (b"minus.plus", true),
  b"div" => (b"div", true),
  b"star" => (b"star", true),
  b"cap" => (b"sect", true),
  b"cup" => (b"union", true),
  b"in" => (b"in", true),
  b"subset" => (b"subset", true),
  b"subseteq" => (b"subset.eq", true),
  b"lt" => (b"<", true),
  b"gt" => (b">", true),
  b"le" => (b"<=", true),
  b"ge" => (b">=", true),
  b"leq" => (b"<=", true),
  b"geq" => (b">=", true),
  b"leqslant" => (b"lt.eq.slant", true),
  b"geqslant" => (b"gt.eq.slant", true),
  b"approx" => (b"approx", true),
  // Hack
  b"over" => (b")/(", false),
  // Accents
  b"not" => (b"cancel", false),
  b"grave" => (b"grave", false),
  b"acute" => (b"acute", false),
  b"hat" => (b"hat", false),
  b"tilde" => (b"tilde", false),
  b"bar" => (b"macron", false),
  b"breve" => (b"breve", false),
  b"dot" => (b"dot", false),
  b"ddot" => (b"dot.double", false),
  b"dddot" => (b"dot.triple", false),
  b"ddddot" => (b"dot.quad", false),
  b"H" => (b"acute.double", false),
  b"v" => (b"caron", false),
  b"vec" => (b"arrow", false),
  b"overrightarrow" => (b"arrow", false),
  b"overleftarrow" => (b"arrow.l", false),
  b"overline" => (b"overline", false),
  b"underline" => (b"underline", false),
  // Styles and variants
  b"mathbf" => (b"bold", false),
  b"mathrm" => (b"upright", false),
  b"mathit" => (b"italic", false),
  b"mathsf" => (b"sans", false),
  b"mathfrak" => (b"mathfrak", false),
  b"mathtt" => (b"mono", false),
  b"mathbb" => (b"bb", false),
  b"mathcal" => (b"cal", false),
};

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
            // COMMAND_MAP[group 1]
            let key = &input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()];
            if let Some((replacement, add_space)) = COMMAND_MAP.get(key) {
              output.extend_from_slice(*replacement);
              if *add_space {
                output.extend_from_slice(b" ");
              }
            } else {
              return Result::Err(String::from(format!("invalid command \"\\{}\" at {}", String::from_utf8_lossy(key), i)));
            }
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
