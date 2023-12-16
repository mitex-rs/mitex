extern crate bitflags;
extern crate core;
extern crate mitex_parser;
extern crate rowan;

use core::fmt;
use std::cell::RefCell;
use std::fmt::Write;

use mitex_parser::parse;
use mitex_parser::syntax::Environment;
use mitex_parser::syntax::GenericCommand;
use rowan::ast::AstNode;

// use bitflags::bitflags;
//
// The `bitflags!` macro generates `struct`s that manage a set of flags.
// bitflags! {
//     /// Represents a set of flags.
//     #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//     struct ConvertContext: u32 {
//         /// The value `Matrix`, at bit position `0`.
//         const Matrix = 0b00000001;

//         /// The combination of `A`, `B`, and `C`.
//         const ABC = Self::Matrix.bits();
//     }
// }

#[derive(Debug, Clone, Copy)]
enum MatrixKind {
    Matrix,
    PMatrix,
    BMatrix,
    BbMatrix,
    VMatrix,
    VvMatrix,
    Array,
}

#[derive(Debug, Clone, Copy, Default)]
enum LaTeXEnv {
    #[default]
    None,
    Matrix(MatrixKind),
}

struct MathConverter {
    env: LaTeXEnv,
}

impl MathConverter {
    fn new() -> Self {
        Self {
            env: LaTeXEnv::default(),
        }
    }

    #[must_use]
    fn enter_env(&mut self, context: LaTeXEnv) -> LaTeXEnv {
        let prev = self.env;
        self.env = context;
        prev
    }

    fn exit_env(&mut self, prev: LaTeXEnv) {
        self.env = prev;
    }
}

// fn empty_node() -> GreenNode {
//     rowan::GreenNode::new(LatexSyntaxKind::TEXT.into(), [])
// }

use mitex_parser::syntax::SyntaxElement as LatexSyntaxElem;
use mitex_parser::syntax::SyntaxKind as LatexSyntaxKind;
// use mitex_parser::syntax::SyntaxNode as LatexSyntaxNode;

#[derive(Debug)]
enum ConvertError {
    Fmt(fmt::Error),
    Str(String),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fmt(e) => write!(f, "fmt: {}", e),
            Self::Str(e) => write!(f, "error: {}", e),
        }
    }
}

impl From<fmt::Error> for ConvertError {
    fn from(e: fmt::Error) -> Self {
        Self::Fmt(e)
    }
}

impl From<String> for ConvertError {
    fn from(e: String) -> Self {
        Self::Str(e)
    }
}

impl MathConverter {
    fn convert(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
    ) -> Result<(), ConvertError> {
        use LatexSyntaxKind::*;

        match elem.kind() {
            ERROR => Err(format!(
                "error unexpected: {:?}",
                elem.as_node().unwrap().text()
            ))?,
            ROOT | PREAMBLE | FORMULA | TEXT | CURLY_GROUP => {
                for child in elem.as_node().unwrap().children_with_tokens() {
                    self.convert(f, child)?;
                }
            }
            COMMAND_NAME => Err("command name outside of command".to_owned())?,
            CURLY_GROUP_WORD | BEGIN | END => Err("clauses outside of environment".to_owned())?,
            WORD => {
                // break up words into individual characters and add a space
                let text = elem.as_token().unwrap().text().to_string();
                let trimed = text.trim();
                if trimed == "&" {
                    if matches!(self.env, LaTeXEnv::Matrix(..)) {
                        f.write_str(",")?;
                    } else {
                        f.write_str("&")?;
                    }
                } else {
                    for prev in text.chars() {
                        f.write_char(prev)?;
                        if !matches!(prev, '^' | '_') {
                            f.write_char(' ')?;
                        }
                    }
                }
            }
            // do nothing
            L_CURLY | R_CURLY | DOLLAR | COMMENT | BLOCK_COMMENT => {}
            // space identical
            LINE_BREAK | WHITESPACE => {
                write!(f, "{}", elem.as_token().unwrap().text())?;
            }
            // paren identical
            L_PAREN | R_PAREN => {
                write!(f, "{}", elem.as_token().unwrap().text())?;
            }
            // escape
            L_BRACK => {
                f.write_str("\\[")?;
            }
            R_BRACK => {
                f.write_str("\\]")?;
            }
            GENERIC_COMMAND => {
                let cmd = GenericCommand::cast(elem.as_node().unwrap().clone()).unwrap();
                let name = cmd.name().unwrap();
                let name = name.text();
                // remove prefix \
                let name = &name[1..];
                // split by _
                let mut split_name = name.split('_');
                let name = split_name.next().unwrap();
                let rest = split_name.collect::<Vec<_>>();
                let rest2 = elem
                    .as_node()
                    .unwrap()
                    .children_with_tokens()
                    .filter(|node| node.kind() != COMMAND_NAME)
                    .collect::<Vec<_>>();
                // println!("name: {:?} with sub {:?} args {:?}", name, rest, rest2);

                let (typst_name, is_fn) = match name {
                    "\\" => {
                        if matches!(self.env, LaTeXEnv::Matrix(..)) {
                            (";", false)
                        } else {
                            ("\\", false)
                        }
                    }

                    // greek letters
                    "alpha" => ("alpha", false),
                    "beta" => ("beta", false),
                    "gamma" => ("gamma", false),
                    "delta" => ("delta", false),
                    "epsilon" => ("epsilon", false),
                    "zeta" => ("zeta", false),
                    "eta" => ("eta", false),
                    "theta" => ("theta", false),
                    "iota" => ("iota", false),
                    "kappa" => ("kappa", false),
                    "lambda" => ("lambda", false),
                    "mu" => ("mu", false),
                    "nu" => ("nu", false),
                    "xi" => ("xi", false),
                    "omicron" => ("omicron", false),
                    "pi" => ("pi", false),
                    "rho" => ("rho", false),
                    "sigma" => ("sigma", false),
                    "tau" => ("tau", false),
                    "upsilon" => ("upsilon", false),
                    "phi" => ("phi", false),
                    "chi" => ("chi", false),
                    "psi" => ("psi", false),
                    "omega" => ("omega", false),
                    "Alpha" => ("Alpha", false),
                    "Beta" => ("Beta", false),
                    "Gamma" => ("Gamma", false),
                    "Delta" => ("Delta", false),
                    "Epsilon" => ("Epsilon", false),
                    "Zeta" => ("Zeta", false),
                    "Eta" => ("Eta", false),
                    "Theta" => ("Theta", false),
                    "Iota" => ("Iota", false),
                    "Kappa" => ("Kappa", false),
                    "Lambda" => ("Lambda", false),
                    "Mu" => ("Mu", false),
                    "Nu" => ("Nu", false),
                    "Xi" => ("Xi", false),
                    "Omicron" => ("Omicron", false),
                    "Pi" => ("Pi", false),
                    "Rho" => ("Rho", false),
                    "Sigma" => ("Sigma", false),
                    "Tau" => ("Tau", false),
                    "Upsilon" => ("Upsilon", false),
                    "Phi" => ("Phi", false),
                    "Chi" => ("Chi", false),
                    "Psi" => ("Psi", false),
                    "Omega" => ("Omega", false),

                    // Symbols
                    "infty" => ("oo", false),
                    "leftarrow" => ("<-", false),
                    "rightarrow" => ("->", false),
                    "leftrightarrow" => ("<->", false),
                    "Leftarrow" => ("arrow.l.double", false),
                    "Rightarrow" => ("=>", false),
                    "Leftrightarrow" => ("<=>", false),
                    "larr" => ("<-", false),
                    "rarr" => ("->", false),
                    "lrarr" => ("<->", false),
                    "lArr" => ("arrow.l.double", false),
                    "rArr" => ("=>", false),
                    "lrArr" => ("<=>", false),
                    "Larr" => ("arrow.l.double", false),
                    "Rarr" => ("=>", false),
                    "Lrarr" => ("<=>", false),
                    "longleftarrow" => ("<--", false),
                    "longrightarrow" => ("-->", false),
                    "longleftrightarrow" => ("<-->", false),
                    "Longleftarrow" => ("<==", false),
                    "Longrightarrow" => ("==>", false),
                    "Longleftrightarrow" => ("<==>", false),
                    "to" => ("->", false),
                    "mapsto" => ("|->", false),
                    // Functions
                    "sin" => ("sin", true),
                    "cos" => ("cos", true),
                    "tan" => ("tan", true),
                    "cot" => ("cot", true),
                    "sec" => ("sec", true),
                    "csc" => ("csc", true),
                    "arcsin" => ("arcsin", true),
                    "arccos" => ("arccos", true),
                    "arctan" => ("arctan", true),
                    "sinh" => ("sinh", true),
                    "cosh" => ("cosh", true),
                    "tanh" => ("tanh", true),
                    "coth" => ("coth", true),
                    "ln" => ("ln", true),
                    "log" => ("log", true),
                    "lg" => ("lg", true),
                    "lim" => ("lim", true),
                    "limsup" => ("limsup", true),
                    "liminf" => ("liminf", true),
                    "hom" => ("hom", true),
                    "det" => ("det", true),
                    "exp" => ("exp", true),
                    "deg" => ("deg", true),
                    "gcd" => ("gcd", true),
                    "lcm" => ("lcm", true),
                    "dim" => ("dim", true),
                    "ker" => ("ker", true),
                    "arg" => ("arg", true),
                    "Pr" => ("Pr", true),
                    // Limits
                    "max" => ("max", true),
                    "min" => ("min", true),
                    "argmax" => ("op(limits: #true, arg max)", true),
                    "argmin" => ("op(limits: #true, arg min)", true),
                    "sup" => ("sup", true),
                    "inf" => ("inf", true),
                    "sum" => ("sum", true),
                    "prod" => ("product", true),
                    // Integrals
                    "int" => ("integral", false),
                    "iint" => ("integral.double", false),
                    "iiint" => ("integral.triple", false),
                    "oint" => ("integral.cont", false),
                    "oiint" => ("integral.surf", false),
                    "oiiint" => ("integral.vol", false),
                    // Operators
                    "mod" => ("mod", false),
                    "cdot" => ("dot.c", false),
                    "times" => ("times", false),
                    "oplus" => ("plus.circle", false),
                    "ominus" => ("minus.circle", false),
                    "pm" => ("plus.minus", false),
                    "mp" => ("minus.plus", false),
                    "div" => ("div", false),
                    "star" => ("star", false),
                    "cap" => ("sect", false),
                    "cup" => ("union", false),
                    "in" => ("in", false),
                    "notin" => ("in.not", false),
                    "subset" => ("subset", false),
                    "subseteq" => ("subset.eq", false),
                    "neq" => ("!=", false),
                    "lt" => ("<", false),
                    "gt" => (">", false),
                    "le" => ("<=", false),
                    "ge" => (">=", false),
                    "leq" => ("<=", false),
                    "geq" => (">=", false),
                    "leqslant" => ("lt.eq.slant", false),
                    "geqslant" => ("gt.eq.slant", false),
                    "approx" => ("approx", false),
                    // todo: Hacks
                    "left" => ("lr(", false),
                    "right" => (")", false),
                    "over" => (")/(", false),

                    // Accents
                    "not" => ("cancel", true),
                    "grave" => ("grave", true),
                    "acute" => ("acute", true),
                    "hat" => ("hat", true),
                    "tilde" => ("tilde", true),
                    "bar" => ("macron", true),
                    "breve" => ("breve", true),
                    "dot" => ("dot", true),
                    "ddot" => ("dot.double", true),
                    "dddot" => ("dot.triple", true),
                    "ddddot" => ("dot.quad", true),
                    "H" => ("acute.double", true),
                    "v" => ("caron", true),
                    "vec" => ("arrow", true),
                    "overrightarrow" => ("arrow", true),
                    "overleftarrow" => ("arrow.l", true),
                    "overline" => ("overline", true),
                    "underline" => ("underline", true),
                    // Styles and variants
                    "bold" => ("bold", true),
                    "mathbf" => ("bold", true),
                    "boldsymbol" => ("bold", true),
                    "mathrm" => ("upright", true),
                    "mathit" => ("italic", true),
                    "mathsf" => ("sans", true),
                    "mathfrak" => ("frak", true),
                    "mathtt" => ("mono", true),
                    "mathbb" => ("bb", true),
                    "mathcal" => ("cal", true),
                    // Functions with no space
                    "frac" => ("frac", true),
                    "cfrac" => ("cfrac", true),
                    "dfrac" => ("dfrac", true),
                    "tfrac" => ("tfrac", true),
                    "binom" => ("binom", true),
                    // Ignores
                    "displaystyle" => ("", false),
                    "textstyle" => ("", false),
                    "scriptstyle" => ("", false),
                    "scriptscriptstyle" => ("", false),
                    "bf" => ("", false),
                    "rm" => ("", false),
                    "it" => ("", false),
                    "sf" => ("", false),
                    "frak" => ("", false),
                    "tt" => ("", false),
                    "cal" => ("", false),
                    "limits" => ("", false),
                    "nolimits" => ("", false),

                    _ => (name, false),
                };

                write!(f, "{}", typst_name)?;

                if !rest.is_empty() {
                    for part in rest {
                        f.write_char('_')?;
                        f.write_str(part)?;
                    }
                }

                if is_fn {
                    f.write_char('(')?;

                    for arg in rest2 {
                        let kind = arg.kind();
                        self.convert(f, arg)?;
                        if matches!(kind, CURLY_GROUP) {
                            f.write_char(',')?;
                        }
                    }

                    f.write_char(')')?;
                } else {
                    f.write_char(' ')?
                }
            }
            ENVIRONMENT => {
                let env = Environment::cast(elem.as_node().unwrap().clone()).unwrap();
                // todo: handle unwraps
                let beg = env.begin().unwrap();
                let name = beg
                    .name()
                    .unwrap()
                    .key()
                    .unwrap()
                    .syntax()
                    .text()
                    .to_string();
                let name = name.trim();
                // todo: handle end
                // todo: handle options

                let env_kind = match name {
                    "matrix" => LaTeXEnv::Matrix(MatrixKind::Matrix),
                    "pmatrix" => LaTeXEnv::Matrix(MatrixKind::PMatrix),
                    "bmatrix" => LaTeXEnv::Matrix(MatrixKind::BMatrix),
                    "Bmatrix" => LaTeXEnv::Matrix(MatrixKind::BbMatrix),
                    "vmatrix" => LaTeXEnv::Matrix(MatrixKind::VMatrix),
                    "Vmatrix" => LaTeXEnv::Matrix(MatrixKind::VvMatrix),
                    "array" => LaTeXEnv::Matrix(MatrixKind::Array),
                    // ignored
                    _ => LaTeXEnv::None,
                };

                match env_kind {
                    LaTeXEnv::Matrix(MatrixKind::Matrix) => {
                        f.write_str(r#"mat(delim: #none,"#)?;
                    }
                    LaTeXEnv::Matrix(MatrixKind::PMatrix) => {
                        f.write_str(r#"mat(delim: "(","#)?;
                    }
                    LaTeXEnv::Matrix(MatrixKind::BMatrix) => {
                        f.write_str(r#"mat(delim: "[","#)?;
                    }
                    LaTeXEnv::Matrix(MatrixKind::BbMatrix) => {
                        f.write_str(r#"mat(delim: "{","#)?;
                    }
                    LaTeXEnv::Matrix(MatrixKind::VMatrix) => {
                        f.write_str(r#"mat(delim: "|","#)?;
                    }
                    LaTeXEnv::Matrix(MatrixKind::VvMatrix) => {
                        f.write_str(r#"mat(delim: "||","#)?;
                    }
                    LaTeXEnv::Matrix(MatrixKind::Array) => {
                        f.write_str(r#"mat(delim: #none,"#)?;
                    }
                    _ => {}
                }

                let prev = self.enter_env(env_kind);

                for child in elem.as_node().unwrap().children_with_tokens() {
                    if matches!(child.kind(), BEGIN | END) {
                        continue;
                    }

                    self.convert(f, child)?;
                }

                if let LaTeXEnv::Matrix(..) = env_kind {
                    f.write_str(r#")"#)?;
                }

                self.exit_env(prev);
            }
            VERBATIM => todo!(),
            COMMA => todo!(),
            EQUALITY_SIGN => todo!(),
            KEY => todo!(),
            CURLY_GROUP_COMMAND => todo!(),
            BRACK_GROUP => todo!(),
            MIXED_GROUP => todo!(),
            EQUATION => todo!(),
            MATH_OPERATOR => todo!(),
            COLOR_REFERENCE => todo!(),
        };

        Ok(())
    }
}

struct TypstMathRepr(LatexSyntaxElem, RefCell<String>);

impl fmt::Display for TypstMathRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ctx = MathConverter::new();
        if let Err(e) = ctx.convert(f, self.0.clone()) {
            self.1.borrow_mut().push_str(&e.to_string());
            return Err(fmt::Error);
        }
        Ok(())
    }
}

pub fn convert_math(input: &str) -> Result<String, String> {
    // let input = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let node = parse(input);
    // println!("{:#?}", node);
    // println!("{:#?}", node.text());
    let mut output = String::new();
    let err = String::new();
    let err = RefCell::new(err);
    let repr = TypstMathRepr(LatexSyntaxElem::Node(node), err.clone());
    core::fmt::write(&mut output, format_args!("{}", repr)).map_err(|_| err.borrow().to_owned())?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_word() {
        assert_eq!(convert_math("$abc$"), Ok("a b c ".to_owned()));
    }

    #[test]
    fn test_convert_command() {
        assert_eq!(
            convert_math("$\\int_1^2 x \\mathrm{d} x$"),
            Ok("integral_1^2  x  upright(d  ,)x ".to_owned())
        );
        assert_eq!(
            convert_math("$\\underline{T}$"),
            Ok("underline(T ,)".to_owned())
        );
    }

    #[test]
    fn test_convert_frac() {
        assert_eq!(
            convert_math("$\\frac{a}{b}$"),
            Ok("frac(a ,b ,)".to_owned())
        );
    }

    #[test]
    fn test_convert_matrix() {
        assert_eq!(
            convert_math(
                r#"$\begin{matrix}
1 & 2 & 3\\
a & b & c
\end{matrix}$"#
            ),
            Ok("mat(delim: #none,1  , 2  , 3 ;a  , b  , c \n)".to_owned())
        );
        assert_eq!(
            convert_math(
                r#"$\begin{Vmatrix}
1 & 2 & 3\\
a & b & c
\end{Vmatrix}$"#
            ),
            Ok(r##"mat(delim: "||",1  , 2  , 3 ;a  , b  , c 
)"##
            .to_owned())
        );
    }
}
