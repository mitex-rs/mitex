extern crate bitflags;
extern crate core;
extern crate mitex_parser;
extern crate rowan;

use core::fmt;
use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

pub use mitex_parser::command_preludes;
use mitex_parser::parse;
pub use mitex_parser::spec::*;
use mitex_parser::syntax::CmdItem;
use mitex_parser::syntax::EnvItem;
use rowan::ast::AstNode;
use rowan::SyntaxToken;

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

#[derive(Debug, Clone, Copy, Default)]
enum LaTeXEnv {
    #[default]
    None,
    SubStack,
    CurlyGroup,
    Matrix,
    Cases,
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
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
        use LatexSyntaxKind::*;

        match elem.kind() {
            TokenError => Err(format!(
                "error unexpected: {:?}",
                elem.as_node().unwrap().text()
            ))?,
            ItemLR | ClauseArgument | ScopeRoot | ItemFormula | ItemText | ItemBracket
            | ItemParen => {
                for child in elem.as_node().unwrap().children_with_tokens() {
                    self.convert(f, child, spec)?;
                }
            }
            ItemCurly => {
                // deal with case like `\begin{pmatrix}x{\\}x\end{pmatrix}`
                let mut prev = LaTeXEnv::None;
                let mut enter_new_env = false;
                // hack for \substack{abc \\ bcd}
                if !matches!(self.env, LaTeXEnv::SubStack) {
                    prev = self.enter_env(LaTeXEnv::CurlyGroup);
                    enter_new_env = true;
                }
                for child in elem.as_node().unwrap().children_with_tokens() {
                    self.convert(f, child, spec)?;
                }
                // here is hack for case like `\color{red} xxx` and `{}_1^2x_3^4`
                f.write_str("zws ")?;
                if enter_new_env {
                    self.exit_env(prev);
                }
            }
            // handle lr
            ClauseLR => {
                let name_and_args = elem
                    .as_node()
                    .unwrap()
                    .children_with_tokens()
                    .collect::<Vec<_>>();
                let name = name_and_args[0].as_token().unwrap().text();
                // remove prefix \
                let name = &name[1..];
                let args = name_and_args[1..].to_owned();
                if name == "left" {
                    f.write_str("lr(")?;
                }
                for arg in args {
                    match arg {
                        LatexSyntaxElem::Node(node) => {
                            self.convert(f, LatexSyntaxElem::Node(node), spec)?;
                        }
                        LatexSyntaxElem::Token(token) => match token.kind() {
                            TokenWord if token.text() == "." => {}
                            _ => self
                                .convert(f, rowan::NodeOrToken::Token(token), spec)
                                .expect("fail to convert token"),
                        },
                    }
                }
                if name == "right" {
                    f.write_char(')')?;
                }
            }
            ItemAttachComponent => {
                let mut based = false;
                let mut first = true;
                for child in elem.as_node().unwrap().children_with_tokens() {
                    if first {
                        let kind = child.as_token().map(|n| n.kind());
                        if matches!(kind, Some(TokenUnderline | TokenCaret)) {
                            if !based {
                                f.write_str("zws")?;
                            }
                            write!(f, "{}(", child.as_token().unwrap().text())?;
                            first = false;
                            continue;
                        } else if !matches!(kind, Some(TokenWhiteSpace)) {
                            based = true;
                        }
                    }
                    self.convert(f, child, spec)?;
                }
                if !first {
                    f.write_char(')')?;
                }
            }
            TokenApostrophe => {
                f.write_char('\'')?;
            }
            ClauseCommandName => Err("command name outside of command".to_owned())?,
            ItemBegin | ItemEnd => Err("clauses outside of environment".to_owned())?,
            TokenWord => {
                // break up words into individual characters and add a space
                let text = elem.as_token().unwrap().text().to_string();
                for prev in text.chars() {
                    f.write_char(prev)?;
                    f.write_char(' ')?;
                }
            }
            // do nothing
            TokenLBrace | TokenRBrace | TokenDollar | TokenComment | ItemBlockComment => {}
            // space identical
            TokenLineBreak | TokenWhiteSpace => {
                write!(f, "{}", elem.as_token().unwrap().text())?;
            }
            // equal identical
            TokenEqual => {
                write!(f, "{}", elem.as_token().unwrap().text())?;
            }
            // escape
            TokenComma => {
                f.write_str("\\,")?;
            }
            TokenTilde => {
                f.write_str("space.nobreak ")?;
            }
            TokenDivide => {
                f.write_str("\\/")?;
            }
            TokenCaret => {
                f.write_str("\\^")?;
            }
            TokenDitto => {
                f.write_str("\\\"")?;
            }
            TokenUnderline => {
                f.write_str("\\_")?;
            }
            TokenLParen => {
                f.write_str("\\(")?;
            }
            TokenRParen => {
                f.write_str("\\)")?;
            }
            TokenLBracket => {
                f.write_str("\\[")?;
            }
            TokenRBracket => {
                f.write_str("\\]")?;
            }
            TokenAnd => match self.env {
                LaTeXEnv::Matrix => f.write_str("zws ,")?,
                _ => f.write_str("&")?,
            },
            ItemNewLine => match self.env {
                LaTeXEnv::Matrix => f.write_str("zws ;")?,
                LaTeXEnv::Cases => f.write_str(",")?,
                LaTeXEnv::CurlyGroup => {}
                _ => f.write_str("\\ ")?,
            },
            // for left/right
            TokenCommandSym => {
                let name = elem.as_token().unwrap().text();
                // remove prefix \
                let name = &name[1..];
                // get cmd_shape and arg_shape from spec
                let cmd_shape = spec
                    .get_cmd(name)
                    .ok_or_else(|| format!("unknown command: \\{}", name))?;
                // typst alias name
                let typst_name = cmd_shape.alias.as_deref().unwrap_or(name);
                // write to output
                write!(f, "{}", typst_name)?;
            }
            ItemCmd => {
                let cmd = CmdItem::cast(elem.as_node().unwrap().clone()).unwrap();
                let name = cmd.name_tok().unwrap();
                let name = name.text();
                // remove prefix \
                let name = &name[1..];
                let args = elem
                    .as_node()
                    .unwrap()
                    .children_with_tokens()
                    .filter(|node| node.kind() != ClauseCommandName)
                    .collect::<Vec<_>>();
                // println!("name: {:?} with args {:?}", name, args);

                // get cmd_shape and arg_shape from spec
                let cmd_shape = spec
                    .get_cmd(name)
                    .ok_or_else(|| format!("unknown command: \\{}", name))?;
                let arg_shape = &cmd_shape.args;
                // typst alias name
                let typst_name = cmd_shape.alias.as_deref().unwrap_or(name);

                if typst_name.starts_with("text") {
                    f.write_str(typst_name)?;
                    f.write_str("(\"")?;

                    fn is_trivia_elem(elem: &LatexSyntaxElem) -> bool {
                        elem.as_token()
                            .map(SyntaxToken::kind)
                            .map_or(false, LatexSyntaxKind::is_trivia)
                    }

                    let mut args = args.as_slice();
                    while args.first().map_or(false, is_trivia_elem) {
                        args = &args[1..];
                    }
                    while args.last().map_or(false, is_trivia_elem) {
                        args = &args[..args.len() - 1];
                    }

                    for arg in args {
                        if let Some(text) = arg.as_token() {
                            if matches!(text.kind(), TokenLBrace | TokenRBrace) {
                                continue;
                            }
                            f.write_str(text.text())?;
                        } else {
                            arg.as_node()
                                .unwrap()
                                .descendants_with_tokens()
                                .for_each(|child| {
                                    if let Some(text) = child.as_token() {
                                        if matches!(text.kind(), TokenLBrace | TokenRBrace) {
                                            return;
                                        }
                                        if matches!(text.kind(), TokenDitto) {
                                            f.write_str("\\\"").unwrap();
                                            return;
                                        }
                                        f.write_str(text.text()).unwrap();
                                    }
                                });
                        }
                    }

                    f.write_str("\")")?;
                    return Ok(());
                }

                write!(f, "{}", typst_name)?;

                // hack for \substack{abc \\ bcd}
                let mut prev = LaTeXEnv::None;
                if typst_name == "substack" {
                    prev = self.enter_env(LaTeXEnv::SubStack);
                }

                if let ArgShape::Right(ArgPattern::None) = arg_shape {
                    f.write_char(' ')?
                } else {
                    f.write_char('(')?;

                    let mut cnt = 0;
                    let args_len = args.len();
                    for arg in args {
                        cnt += 1;
                        let kind = arg.kind();
                        self.convert(f, arg, spec)?;
                        if matches!(kind, ClauseArgument) && cnt != args_len {
                            f.write_char(',')?;
                        }
                    }

                    f.write_char(')')?;
                }

                // hack for \substack{abc \\ bcd}
                if typst_name == "substack" {
                    self.exit_env(prev);
                }
            }
            ItemEnv => {
                let env = EnvItem::cast(elem.as_node().unwrap().clone()).unwrap();
                let name = env
                    .name_tok()
                    .expect("environment name must be non-empty")
                    .text()
                    .to_string();
                let name = name.trim();
                let args = env.arguments();
                // todo: handle options

                let env_shape = spec
                    .get_env(name)
                    .ok_or_else(|| format!("unknown environment: \\{}", name))?;
                let typst_name = env_shape.alias.as_deref().unwrap_or(name);

                let env_kind = match env_shape.ctx_feature {
                    ContextFeature::None => LaTeXEnv::None,
                    ContextFeature::IsMatrix => LaTeXEnv::Matrix,
                    ContextFeature::IsCases => LaTeXEnv::Cases,
                };

                // environment name
                f.write_str(typst_name)?;
                f.write_char('(')?;
                // named args
                for (index, arg) in args.enumerate() {
                    f.write_str(format!("arg{}: ", index).as_str())?;
                    self.convert(f, rowan::NodeOrToken::Node(arg), spec)?;
                    f.write_char(',')?;
                }

                let prev = self.enter_env(env_kind);

                for child in elem.as_node().unwrap().children_with_tokens() {
                    if matches!(child.kind(), ItemBegin | ItemEnd) {
                        continue;
                    }

                    self.convert(f, child, spec)?;
                }

                f.write_char(')')?;

                self.exit_env(prev);
            }
        };

        Ok(())
    }
}

struct TypstMathRepr(LatexSyntaxElem, CommandSpec, Rc<RefCell<String>>);

impl fmt::Display for TypstMathRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ctx = MathConverter::new();
        if let Err(e) = ctx.convert(f, self.0.clone(), &self.1) {
            self.2.borrow_mut().push_str(&e.to_string());
            return Err(fmt::Error);
        }
        Ok(())
    }
}

pub fn convert_math(input: &str, spec: Option<CommandSpec>) -> Result<String, String> {
    let node = parse(input, spec.unwrap_or_else(|| DEFAULT_SPEC.clone()));
    // println!("{:#?}", node);
    // println!("{:#?}", node.text());
    let mut output = String::new();
    let err = String::new();
    let err = Rc::new(RefCell::new(err));
    let repr = TypstMathRepr(
        LatexSyntaxElem::Node(node),
        DEFAULT_SPEC.clone(),
        err.clone(),
    );
    core::fmt::write(&mut output, format_args!("{}", repr)).map_err(|_| err.borrow().to_owned())?;
    Ok(output)
}

static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(|| {
    CommandSpec::from_bytes(include_bytes!(
        "../../../target/mitex-artifacts/spec/default.rkyv"
    ))
});

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    fn convert_math(input: &str) -> Result<String, String> {
        crate::convert_math(input, None)
    }

    #[test]
    fn test_convert_word() {
        assert_debug_snapshot!(convert_math(r#"$abc$"#), @r###"
        Ok(
            "a b c ",
        )
        "###);
    }

    #[test]
    fn test_convert_greek() {
        assert_debug_snapshot!(convert_math(r#"$\alpha x$"#), @r###"
        Ok(
            "alpha  x ",
        )
        "###);
    }

    #[test]
    fn test_convert_command() {
        assert_debug_snapshot!(convert_math(r#"$\int_1^2 x \mathrm{d} x$"#), @r###"
        Ok(
            "integral _(1 )^(2 ) x  upright(d  zws )x ",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$\underline{T}$"#), @r###"
        Ok(
            "underline(T zws )",
        )
        "###);
    }

    #[test]
    fn test_convert_frac() {
        assert_debug_snapshot!(convert_math(r#"$\frac{a}{b}$"#), @r###"
        Ok(
            "frac(a zws ,b zws )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$\frac 12_3$"#), @r###"
        Ok(
            "frac( 1 ,2 )_(3 )",
        )
        "###
        );
        // Note: the following is invalid in TeX, hence we may output anything.
        let _ = convert_math(r#"$\frac a_c b$"#);
    }

    #[test]
    fn test_convert_displaystyle() {
        assert_debug_snapshot!(convert_math(r#"$\displaystyle xyz\frac{1}{2}$"#), @r###"
        Ok(
            "mitexdisplay( x y z frac(1 zws ,2 zws ))",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$1 + {\displaystyle 23} + 4$"#), @r###"
        Ok(
            "1  +  mitexdisplay( 2 3 ) zws +  4 ",
        )
        "###
        );
    }

    #[test]
    fn test_convert_limits() {
        assert_debug_snapshot!(convert_math(r#"$\sum\limits_1^2$"#), @r###"
        Ok(
            "limits(sum )_(1 )^(2 )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_subsup() {
        assert_debug_snapshot!(convert_math(r#"$x_1^2$"#), @r###"
        Ok(
            "x _(1 )^(2 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$x^2_1$"#), @r###"
        Ok(
            "x ^(2 )_(1 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$x''_1$"#), @r###"
        Ok(
            "x ''_(1 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$\overbrace{a + b + c}^{\text{This is an overbrace}}$"#), @r###"
        Ok(
            "mitexoverbrace(a  +  b  +  c zws )^(text(\"This is an overbrace\")zws )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$x_1''$"#), @r###"
        Ok(
            "x _(1 )''",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"${}_1^2x_3^4$"#), @r###"
        Ok(
            "zws _(1 )^(2 )x _(3 )^(4 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$_1^2$"#), @r###"
        Ok(
            "zws_(1 )zws^(2 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$\frac{_1^2}{3}$"#), @r###"
        Ok(
            "frac(zws_(1 )zws^(2 )zws ,3 zws )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_over() {
        assert_debug_snapshot!(convert_math(r#"$x + 1 \over y + 2$"#), @r###"
        Ok(
            "frac(x  +  1  , y  +  2 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$1 + {2 \over 3}$"#), @r###"
        Ok(
            "1  +  frac(2  , 3 )zws ",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"${l \over 2'}$"#), @r###"
        Ok(
            "frac(l  , 2 ')zws ",
        )
        "###);
    }

    #[test]
    fn test_convert_divide() {
        assert_debug_snapshot!(convert_math(r#"$x / y$"#), @r###"
        Ok(
            "x  \\/ y ",
        )
        "###
        );
    }

    #[test]
    fn test_convert_space() {
        assert_debug_snapshot!(convert_math(r#"$x~\! \, \> \: \; \ \quad \qquad y$"#), @r###"
        Ok(
            "x space.nobreak negthinspace  thin  med  med  thick  thick  quad  wide  y ",
        )
        "###
        );
    }

    #[test]
    fn test_convert_escape() {
        assert_debug_snapshot!(convert_math(r#"$\|x\|| \& \# \% \$ y$"#), @r###"
        Ok(
            "|| x || |  amp  hash  percent  dollar  y ",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$"$"#).unwrap(), @r###""\\\"""###
        );
    }

    #[test]
    fn test_unreachable() {
        // println!("{:#?}", convert_math(r#"$u^-$"#));
        assert_debug_snapshot!(convert_math(r#"$u^−$"#).unwrap(), @r###""u ^(− )""###
        );
    }

    #[test]
    fn test_convert_sqrt() {
        assert_debug_snapshot!(convert_math(r#"$\sqrt 1$"#), @r###"
        Ok(
            "mitexsqrt( 1 )",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$\sqrt [1]2$"#), @r###"
        Ok(
            "mitexsqrt( \\[1 \\],2 )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_lr() {
        assert_debug_snapshot!(convert_math(r#"$\left.\right.$"#), @r###"
        Ok(
            "lr()",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$\left.a\right.$"#), @r###"
        Ok(
            "lr(a )",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$\alpha\left.\right.$"#), @r###"
        Ok(
            "alpha lr()",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$\left  . a \right    \|$"#), @r###"
        Ok(
            "lr(   a      ||)",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$\left\langle a\right\|$"#), @r###"
        Ok(
            "lr(angle.l a ||)",
        )
        "###
        );
    }

    #[test]
    fn test_convert_color() {
        assert_debug_snapshot!(convert_math(r#"$x\color{red}yz\frac{1}{2}$"#), @r###"
        Ok(
            "x mitexcolor(r e d zws y z frac(1 zws ,2 zws ))",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$x\textcolor{red}yz$"#), @r###"
        Ok(
            "x colortext(r e d zws ,y )z ",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$x\textcolor{red}{yz}$"#), @r###"
        Ok(
            "x colortext(r e d zws ,y z zws )",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$x\colorbox{red}yz$"#), @r###"
        Ok(
            "x colorbox(r e d zws ,y )z ",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$x\colorbox{red}{yz}$"#), @r###"
        Ok(
            "x colorbox(r e d zws ,y z zws )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_matrix() {
        assert_debug_snapshot!(convert_math(
                     r#"$\begin{pmatrix}x{\\}x\end{pmatrix}$"#
            ).unwrap(),
            @r###""pmatrix(x zws x )""###
        );
        assert_debug_snapshot!(convert_math(
                     r#"$\begin{pmatrix} \\ & \ddots \end{pmatrix}$"#
            ).unwrap(),
            @r###""pmatrix(zws ; zws , dots.down  )""###
        );
        assert_debug_snapshot!(convert_math(
                r#"$\begin{matrix}
        1 & 2 & 3\\
a & b & c
\end{matrix}$"#
            ),
            @r###"
        Ok(
            "matrix(1  zws , 2  zws , 3 zws ;\na  zws , b  zws , c \n)",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(
                r#"$\begin{Vmatrix}
        1 & 2 & 3\\
a & b & c
\end{Vmatrix}$"#
            ),
            @r###"
        Ok(
            "Vmatrix(1  zws , 2  zws , 3 zws ;\na  zws , b  zws , c \n)",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(
                r#"$\begin{array}{lcr}
        1 & 2 & 3\\
a & b & c
\end{array}$"#
            ),
            @r###"
        Ok(
            "mitexarray(arg0: l c r \n        zws ,1  zws , 2  zws , 3 zws ;\na  zws , b  zws , c \n)",
        )
        "###
        );
    }

    #[test]
    fn test_convert_env() {
        assert_debug_snapshot!(convert_math(
                r#"$\begin{aligned}
        1 & 2 & 3\\
a & b & c
\end{aligned}$"#
            ),
            @r###"
        Ok(
            "aligned(1  & 2  & 3 \\ \na  & b  & c \n)",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(
                r#"$\begin{align*}
        1 & 2 & 3\\
a & b & c
\end{align*}$"#
            ),
            @r###"
        Ok(
            "aligned(1  & 2  & 3 \\ \na  & b  & c \n)",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(
                r#"$\begin{cases}
        1 & 2 & 3\\
a & b & c
\end{cases}$"#
            ),
            @r###"
        Ok(
            "cases(1  & 2  & 3 ,\na  & b  & c \n)",
        )
        "###
        );
    }

    #[test]
    fn test_convert_ditto() {
        assert_debug_snapshot!(convert_math(r#"$"$"#).unwrap(), @r###""\\\"""###);
        assert_debug_snapshot!(convert_math(r#"$a"b"c$"#).unwrap(), @r###""a \" b \" c ""###);
        assert_debug_snapshot!(convert_math(r#"$\text{a"b"c}$"#).unwrap(), @r###""text(\"a\"b\"c\")""###);
        assert_debug_snapshot!(convert_math(r#"$\text{a " b " c}$"#).unwrap(), @r###""text(\"a \\\" b \\\" c\")""###);     
    }
    
    #[test]
    fn test_convert_text() {
        assert_debug_snapshot!(convert_math(r#"$\text{abc}$"#).unwrap(), @r###""text(\"abc\")""###);
        assert_debug_snapshot!(convert_math(r#"$\text{ a b c }$"#).unwrap(), @r###""text(\" a b c \")""###);
        assert_debug_snapshot!(convert_math(r#"$\text{abc{}}$"#).unwrap(), @r###""text(\"abc\")""###);
        assert_debug_snapshot!(convert_math(r#"$\text{ab{}c}$"#).unwrap(), @r###""text(\"abc\")""###);
        assert_debug_snapshot!(convert_math(r#"$\text{ab c}$"#).unwrap(), @r###""text(\"ab c\")""###);
        // note: hack doesn't work in this case
        assert_debug_snapshot!(convert_math(r#"$\text{ab\color{red}c}$"#).unwrap(), @r###""text(\"ab\\colorredc\")""###);
    }
}
