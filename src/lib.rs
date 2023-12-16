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
  Text,  // text mode, like \text and \operatorname
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
  // Hack for {\displaystyle ...} and {\rm ...}
  (Regex::new(r"^\{[ \n]*\\displaystyle[ \n]").unwrap(), b"display(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\textstyle[ \n]").unwrap(), b"inline(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\scriptstyle[ \n]").unwrap(), b"script(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\scriptscriptstyle[ \n]").unwrap(), b"sscript(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\bf[ \n]").unwrap(), b"bold(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\rm[ \n]").unwrap(), b"upright(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\it[ \n]").unwrap(), b"italic(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\sf[ \n]").unwrap(), b"sans(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\frak[ \n]").unwrap(), b"frak(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\tt[ \n]").unwrap(), b"mono(", MapType::Normal),
  (Regex::new(r"^\{[ \n]*\\cal[ \n]").unwrap(), b"cal(", MapType::Normal),
  (Regex::new(r"^\{").unwrap(), b"(", MapType::NoSpace),
  // Just a hack for "}{" in "frac{}{}"
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
  // Limits and scripts
  (Regex::new(r"^\\max[ \n]*\\limits").unwrap(), b"limits(max)", MapType::NoSpace),
  (Regex::new(r"^\\min[ \n]*\\limits").unwrap(), b"limits(min)", MapType::NoSpace),
  (Regex::new(r"^\\argmax[ \n]*\\limits").unwrap(), b"limits(arg max)", MapType::NoSpace),
  (Regex::new(r"^\\argmin[ \n]*\\limits").unwrap(), b"limits(arg min)", MapType::NoSpace),
  (Regex::new(r"^\\sup[ \n]*\\limits").unwrap(), b"limits(sup)", MapType::NoSpace),
  (Regex::new(r"^\\inf[ \n]*\\limits").unwrap(), b"limits(inf)", MapType::NoSpace),
  (Regex::new(r"^\\sum[ \n]*\\limits").unwrap(), b"limits(sum)", MapType::NoSpace),
  (Regex::new(r"^\\prod[ \n]*\\limits").unwrap(), b"limits(prod)", MapType::NoSpace),
  (Regex::new(r"^\\int[ \n]*\\limits").unwrap(), b"limits(int)", MapType::NoSpace),
  (Regex::new(r"^\\max[ \n]*\\nolimits").unwrap(), b"scripts(max)", MapType::NoSpace),
  (Regex::new(r"^\\min[ \n]*\\nolimits").unwrap(), b"scripts(min)", MapType::NoSpace),
  (Regex::new(r"^\\argmax[ \n]*\\nolimits").unwrap(), b"scripts(arg max)", MapType::NoSpace),
  (Regex::new(r"^\\argmin[ \n]*\\nolimits").unwrap(), b"scripts(arg min)", MapType::NoSpace),
  (Regex::new(r"^\\sup[ \n]*\\nolimits").unwrap(), b"scripts(sup)", MapType::NoSpace),
  (Regex::new(r"^\\inf[ \n]*\\nolimits").unwrap(), b"scripts(inf)", MapType::NoSpace),
  (Regex::new(r"^\\sum[ \n]*\\nolimits").unwrap(), b"scripts(sum)", MapType::NoSpace),
  (Regex::new(r"^\\prod[ \n]*\\nolimits").unwrap(), b"scripts(prod)", MapType::NoSpace),
  (Regex::new(r"^\\int[ \n]*\\nolimits").unwrap(), b"scripts(int)", MapType::NoSpace),
  // Sqrt
  (Regex::new(r"^\\sqrt[ \n]*\[[ \n]*([0-9]+)[ \n]*\][ \n]*\{").unwrap(), b"", MapType::SqrtN),
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
  // Text
  (Regex::new(r"^\\text[ \n]*\{([^\}]+)\}").unwrap(), b"", MapType::Text),
  (Regex::new(r"^\\operatorname[ \n]*\{([^\}]+)\}").unwrap(), b"op(", MapType::Text),
  (Regex::new(r"^\\operatorname\*[ \n]*\{([^\}]+)\}").unwrap(), b"op(limits: #false, ", MapType::Text),
  (Regex::new(r"^\\operatornamewithlimits[ \n]*\{([^\}]+)\}").unwrap(), b"op(limits: #false, ", MapType::Text),
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
  b"leftarrow" => (b"<-", true),
  b"rightarrow" => (b"->", true),
  b"leftrightarrow" => (b"<->", true),
  b"Leftarrow" => (b"arrow.l.double", true),
  b"Rightarrow" => (b"=>", true),
  b"Leftrightarrow" => (b"<=>", true),
  b"larr" => (b"<-", true),
  b"rarr" => (b"->", true),
  b"lrarr" => (b"<->", true),
  b"lArr" => (b"arrow.l.double", true),
  b"rArr" => (b"=>", true),
  b"lrArr" => (b"<=>", true),
  b"Larr" => (b"arrow.l.double", true),
  b"Rarr" => (b"=>", true),
  b"Lrarr" => (b"<=>", true),
  b"longleftarrow" => (b"<--", true),
  b"longrightarrow" => (b"-->", true),
  b"longleftrightarrow" => (b"<-->", true),
  b"Longleftarrow" => (b"<==", true),
  b"Longrightarrow" => (b"==>", true),
  b"Longleftrightarrow" => (b"<==>", true),
  b"to" => (b"->", true),
  b"mapsto" => (b"|->", true),
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
  b"hom" => (b"hom", true),
  b"det" => (b"det", true),
  b"exp" => (b"exp", true),
  b"deg" => (b"deg", true),
  b"gcd" => (b"gcd", true),
  b"lcm" => (b"lcm", true),
  b"dim" => (b"dim", true),
  b"ker" => (b"ker", true),
  b"arg" => (b"arg", true),
  b"Pr" => (b"Pr", true),
  // Limits
  b"max" => (b"max", true),
  b"min" => (b"min", true),
  b"argmax" => (b"op(limits: #true, arg max)", true),
  b"argmin" => (b"op(limits: #true, arg min)", true),
  b"sup" => (b"sup", true),
  b"inf" => (b"inf", true),
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
  b"notin" => (b"in.not", true),
  b"subset" => (b"subset", true),
  b"subseteq" => (b"subset.eq", true),
  b"neq" => (b"!=", true),
  b"lt" => (b"<", true),
  b"gt" => (b">", true),
  b"le" => (b"<=", true),
  b"ge" => (b">=", true),
  b"leq" => (b"<=", true),
  b"geq" => (b">=", true),
  b"leqslant" => (b"lt.eq.slant", true),
  b"geqslant" => (b"gt.eq.slant", true),
  b"approx" => (b"approx", true),
  // Hacks
  b"left" => (b"lr(", false),
  b"right" => (b")", false),
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
  b"bold" => (b"bold", false),
  b"mathbf" => (b"bold", false),
  b"boldsymbol" => (b"bold", false),
  b"mathrm" => (b"upright", false),
  b"mathit" => (b"italic", false),
  b"mathsf" => (b"sans", false),
  b"mathfrak" => (b"frak", false),
  b"mathtt" => (b"mono", false),
  b"mathbb" => (b"bb", false),
  b"mathcal" => (b"cal", false),
  // Functions with no space
  b"frac" => (b"frac", false),
  b"cfrac" => (b"cfrac", false),
  b"dfrac" => (b"dfrac", false),
  b"tfrac" => (b"tfrac", false),
  b"binom" => (b"binom", false),
  // Ignores
  b"displaystyle" => (b"", false),
  b"textstyle" => (b"", false),
  b"scriptstyle" => (b"", false),
  b"scriptscriptstyle" => (b"", false),
  b"bf" => (b"", false),
  b"rm" => (b"", false),
  b"it" => (b"", false),
  b"sf" => (b"", false),
  b"frak" => (b"", false),
  b"tt" => (b"", false),
  b"cal" => (b"", false),
  b"limits" => (b"", false),
  b"nolimits" => (b"", false),
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
          MapType::Text => {
            // group 1
            let text = &input[i+m.get(1).unwrap().start()..i+m.get(1).unwrap().end()];
            if replacement != b"" {
              // \text{}
              output.extend_from_slice(replacement);
              output.extend_from_slice(b"\"");
              output.extend_from_slice(text);
              output.extend_from_slice(b"\"");
              output.extend_from_slice(b")");
            } else {
              // \operatorname{}
              output.extend_from_slice(b"\"");
              output.extend_from_slice(text);
              output.extend_from_slice(b"\"");
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
