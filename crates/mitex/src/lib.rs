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
            ItemLR | ClauseArgument | ScopeRoot | ItemFormula | ItemText | ItemCurly
            | ItemBracket | ItemParen => {
                for child in elem.as_node().unwrap().children_with_tokens() {
                    self.convert(f, child, spec)?;
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
                        LatexSyntaxElem::Token(token) => {
                            match token.kind() {
                                TokenWord if token.text() == "." => {},
                                _ => self.convert(f, rowan::NodeOrToken::Token(token), spec).expect("fail to convert token"),
                            }
                        }
                    }
                }
                if name == "right" {
                    f.write_char(')')?;
                }
            }
            ItemAttachComponent => {
                let mut first = true;
                // hack for {}_1^2
                write!(f, "zws ")?;
                for child in elem.as_node().unwrap().children_with_tokens() {
                    if first {
                        let kind = child.as_token().map(|n| n.kind());
                        if matches!(kind, Some(TokenUnderline | TokenCaret)) {
                            write!(f, "{}(", child.as_token().unwrap().text())?;
                            first = false;
                            continue;
                        }
                    }
                    self.convert(f, child, spec)?;
                }
                if !first {
                    f.write_char(')')?;
                }
            }
            ClauseCommandName => Err("command name outside of command".to_owned())?,
            ItemBegin | ItemEnd => Err("clauses outside of environment".to_owned())?,
            ClauseArgKey => Err("clauses outside of group".to_owned())?,
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
                f.write_str("med ")?;
            }
            TokenDivide => {
                f.write_str("\\/")?;
            }
            TokenCaret => {
                f.write_str("\\^")?;
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
            TokenAnd => {
                if matches!(self.env, LaTeXEnv::Matrix) {
                    f.write_str(",")?;
                } else if matches!(self.env, LaTeXEnv::Cases) {
                    f.write_str("&")?;
                } else {
                    f.write_str("&")?;
                }
            }
            ItemNewLine => {
                if matches!(self.env, LaTeXEnv::Matrix) {
                    f.write_char(';')?;
                } else if matches!(self.env, LaTeXEnv::Cases) {
                    f.write_char(',')?;
                } else {
                    f.write_char('\\')?;
                }
            }
            // for left/right
            TokenCommandSym => {
                let name = elem.as_token().unwrap().text();
                // remove prefix \
                let name = &name[1..];
                // get cmd_shape and arg_shape from spec
                let cmd_shape = spec
                    .get_cmd(name)
                    .expect(format!("unknown command: \\{}", name).as_str());
                // typst alias name
                let typst_name = cmd_shape.alias.as_deref().unwrap_or(name);
                // write to output
                write!(f, "{}", typst_name)?;
            },
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
                    .expect(format!("unknown command: \\{}", name).as_str());
                let arg_shape = &cmd_shape.args;
                // typst alias name
                let typst_name = cmd_shape.alias.as_deref().unwrap_or(name);

                if typst_name == "text" {
                    f.write_str("#[")?;

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
                                        f.write_str(text.text()).unwrap();
                                    }
                                });
                        }
                    }

                    f.write_char(']')?;
                    return Ok(());
                }

                write!(f, "{}", typst_name)?;

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
            }
            ItemEnv => {
                let env = EnvItem::cast(elem.as_node().unwrap().clone()).unwrap();
                let name = env.name_tok().expect("environment name must be non-empty").text().to_string();
                let name = name.trim();
                let args = env.arguments();
                // todo: handle options

                let env_shape = spec
                    .get_env(name)
                    .expect(format!("unknown environment: \\{}", name).as_str());
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

pub fn convert_math(input: &str) -> Result<String, String> {
    // let input = std::str::from_utf8(input).map_err(|e| e.to_string())?;
    let node = parse(input, DEFAULT_SPEC.clone());
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

fn default_spec() -> CommandSpec {
    use mitex_parser::command_preludes::*;
    let mut builder = SpecBuilder::default();
    // Spaces: \! \, \> \: \; \ \quad \qquad
    builder.add_command("!", define_symbol("negativespace"));
    builder.add_command(",", define_symbol("thin"));
    builder.add_command(">", define_symbol("med"));
    builder.add_command(":", define_symbol("med"));
    builder.add_command(";", define_symbol("thick"));
    builder.add_command("", define_symbol("thick"));
    builder.add_command("quad", TEX_SYMBOL);
    builder.add_command("qquad", define_symbol("wide"));
    // Escape symbols
    builder.add_command("_", define_symbol("\\_"));
    builder.add_command("^", define_symbol("hat"));
    builder.add_command("|", define_symbol("||"));
    builder.add_command("&", define_symbol("amp"));
    builder.add_command("#", define_symbol("hash"));
    builder.add_command("%", define_symbol("percent"));
    builder.add_command("$", define_symbol("dollar"));
    builder.add_command("{", define_symbol("{"));
    builder.add_command("}", define_symbol("}"));
    // Sizes and styles
    builder.add_command("displaystyle", define_greedy_command("mitexdisplay"));
    builder.add_command("textstyle", define_greedy_command("mitexinline"));
    builder.add_command("scriptstyle", define_greedy_command("mitexscript"));
    builder.add_command("scriptscriptstyle", define_greedy_command("mitexsscript"));
    builder.add_command("bf", define_greedy_command("mitexbold"));
    builder.add_command("rm", define_greedy_command("mitexupright"));
    builder.add_command("it", define_greedy_command("mitexitalic"));
    builder.add_command("sf", define_greedy_command("mitexsans"));
    builder.add_command("frak", define_greedy_command("mitexfrak"));
    builder.add_command("tt", define_greedy_command("mitexmono"));
    builder.add_command("cal", define_greedy_command("mitexcal"));
    builder.add_command("bold", define_command_with_alias(1, "bold"));
    builder.add_command("mathbf", define_command_with_alias(1, "bold"));
    builder.add_command("boldsymbol", define_command_with_alias(1, "bold"));
    builder.add_command("mathrm", define_command_with_alias(1, "upright"));
    builder.add_command("mathit", define_command_with_alias(1, "italic"));
    builder.add_command("mathsf", define_command_with_alias(1, "sans"));
    builder.add_command("mathfrak", define_command_with_alias(1, "frak"));
    builder.add_command("mathtt", define_command_with_alias(1, "mono"));
    builder.add_command("mathbb", define_command_with_alias(1, "bb"));
    builder.add_command("mathcal", define_command_with_alias(1, "cal"));
    builder.add_command("color", define_greedy_command("mitexcolor"));
    builder.add_command("textcolor", TEX_CMD2);
    builder.add_command("colorbox", TEX_CMD2);
    builder.add_command("limits", TEX_LEFT1_OPEARTOR);
    builder.add_command(
        "nolimits",
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Left1,
            alias: Some("scripts".to_owned()),
        }),
    );
    // Commands
    builder.add_command("frac", define_command(2));
    builder.add_command("cfrac", define_command(2));
    builder.add_command("dfrac", define_command(2));
    builder.add_command("tfrac", define_command(2));
    builder.add_command("binom", define_command(2));
    builder.add_command("stackrel", define_command(2));
    builder.add_command("overset", define_command(2));
    builder.add_command("underset", define_command(2));
    // Accents
    builder.add_command("not", define_command_with_alias(1, "cancel"));
    builder.add_command("grave", define_command_with_alias(1, "grave"));
    builder.add_command("acute", define_command_with_alias(1, "acute"));
    builder.add_command("hat", define_command_with_alias(1, "hat"));
    builder.add_command("tilde", define_command_with_alias(1, "tilde"));
    builder.add_command("bar", define_command_with_alias(1, "macron"));
    builder.add_command("breve", define_command_with_alias(1, "breve"));
    builder.add_command("dot", define_command_with_alias(1, "dot"));
    builder.add_command("ddot", define_command_with_alias(1, "dot.double"));
    builder.add_command("dddot", define_command_with_alias(1, "dot.triple"));
    builder.add_command("ddddot", define_command_with_alias(1, "dot.quad"));
    builder.add_command("H", define_command_with_alias(1, "acute.double"));
    builder.add_command("v", define_command_with_alias(1, "caron"));
    builder.add_command("vec", define_command_with_alias(1, "arrow"));
    builder.add_command("overrightarrow", define_command_with_alias(1, "arrow"));
    builder.add_command("overleftarrow", define_command_with_alias(1, "arrow.l"));
    builder.add_command("overline", TEX_CMD1);
    builder.add_command("underline", TEX_CMD1);
    // Greeks
    builder.add_command("alpha", TEX_SYMBOL);
    builder.add_command("beta", TEX_SYMBOL);
    builder.add_command("gamma", TEX_SYMBOL);
    builder.add_command("delta", TEX_SYMBOL);
    builder.add_command("epsilon", TEX_SYMBOL);
    builder.add_command("zeta", TEX_SYMBOL);
    builder.add_command("eta", TEX_SYMBOL);
    builder.add_command("theta", TEX_SYMBOL);
    builder.add_command("iota", TEX_SYMBOL);
    builder.add_command("kappa", TEX_SYMBOL);
    builder.add_command("lambda", TEX_SYMBOL);
    builder.add_command("mu", TEX_SYMBOL);
    builder.add_command("nu", TEX_SYMBOL);
    builder.add_command("xi", TEX_SYMBOL);
    builder.add_command("omicron", TEX_SYMBOL);
    builder.add_command("pi", TEX_SYMBOL);
    builder.add_command("rho", TEX_SYMBOL);
    builder.add_command("sigma", TEX_SYMBOL);
    builder.add_command("tau", TEX_SYMBOL);
    builder.add_command("upsilon", TEX_SYMBOL);
    builder.add_command("phi", TEX_SYMBOL);
    builder.add_command("chi", TEX_SYMBOL);
    builder.add_command("psi", TEX_SYMBOL);
    builder.add_command("omega", TEX_SYMBOL);
    builder.add_command("Alpha", TEX_SYMBOL);
    builder.add_command("Beta", TEX_SYMBOL);
    builder.add_command("Gamma", TEX_SYMBOL);
    builder.add_command("Delta", TEX_SYMBOL);
    builder.add_command("Epsilon", TEX_SYMBOL);
    builder.add_command("Zeta", TEX_SYMBOL);
    builder.add_command("Eta", TEX_SYMBOL);
    builder.add_command("Theta", TEX_SYMBOL);
    builder.add_command("Iota", TEX_SYMBOL);
    builder.add_command("Kappa", TEX_SYMBOL);
    builder.add_command("Lambda", TEX_SYMBOL);
    builder.add_command("Mu", TEX_SYMBOL);
    builder.add_command("Nu", TEX_SYMBOL);
    builder.add_command("Xi", TEX_SYMBOL);
    builder.add_command("Omicron", TEX_SYMBOL);
    builder.add_command("Pi", TEX_SYMBOL);
    builder.add_command("Rho", TEX_SYMBOL);
    builder.add_command("Sigma", TEX_SYMBOL);
    builder.add_command("Tau", TEX_SYMBOL);
    builder.add_command("Upsilon", TEX_SYMBOL);
    builder.add_command("Phi", TEX_SYMBOL);
    builder.add_command("Chi", TEX_SYMBOL);
    builder.add_command("Psi", TEX_SYMBOL);
    builder.add_command("Omega", TEX_SYMBOL);
    builder.add_command("varepsilon", TEX_SYMBOL);
    builder.add_command("varphi", TEX_SYMBOL);
    builder.add_command("varpi", TEX_SYMBOL);
    builder.add_command("varrho", TEX_SYMBOL);
    builder.add_command("varsigma", TEX_SYMBOL);
    builder.add_command("vartheta", TEX_SYMBOL);
    builder.add_command("ell", TEX_SYMBOL);
    // Function symbols
    builder.add_command("sin", TEX_SYMBOL);
    builder.add_command("cos", TEX_SYMBOL);
    builder.add_command("tan", TEX_SYMBOL);
    builder.add_command("cot", TEX_SYMBOL);
    builder.add_command("sec", TEX_SYMBOL);
    builder.add_command("csc", TEX_SYMBOL);
    builder.add_command("arcsin", TEX_SYMBOL);
    builder.add_command("arccos", TEX_SYMBOL);
    builder.add_command("arctan", TEX_SYMBOL);
    builder.add_command("sinh", TEX_SYMBOL);
    builder.add_command("cosh", TEX_SYMBOL);
    builder.add_command("tanh", TEX_SYMBOL);
    builder.add_command("coth", TEX_SYMBOL);
    builder.add_command("ln", TEX_SYMBOL);
    builder.add_command("log", TEX_SYMBOL);
    builder.add_command("lg", TEX_SYMBOL);
    builder.add_command("lim", TEX_SYMBOL);
    builder.add_command("limsup", TEX_SYMBOL);
    builder.add_command("liminf", TEX_SYMBOL);
    builder.add_command("hom", TEX_SYMBOL);
    builder.add_command("det", TEX_SYMBOL);
    builder.add_command("exp", TEX_SYMBOL);
    builder.add_command("deg", TEX_SYMBOL);
    builder.add_command("gcd", TEX_SYMBOL);
    builder.add_command("lcm", TEX_SYMBOL);
    builder.add_command("dim", TEX_SYMBOL);
    builder.add_command("ker", TEX_SYMBOL);
    builder.add_command("arg", TEX_SYMBOL);
    builder.add_command("Pr", TEX_SYMBOL);
    // Limits
    builder.add_command("max", TEX_SYMBOL);
    builder.add_command("min", TEX_SYMBOL);
    builder.add_command("argmax", TEX_SYMBOL);
    builder.add_command("argmin", TEX_SYMBOL);
    builder.add_command("sup", TEX_SYMBOL);
    builder.add_command("inf", TEX_SYMBOL);
    builder.add_command("sum", TEX_SYMBOL);
    builder.add_command("prod", define_symbol("product"));
    builder.add_command("int", define_symbol("integral"));
    builder.add_command("iint", define_symbol("integral.double"));
    builder.add_command("iiint", define_symbol("integral.triple"));
    builder.add_command("oint", define_symbol("integral.cont"));
    builder.add_command("oiint", define_symbol("integral.surf"));
    builder.add_command("oiiint", define_symbol("integral.vol"));
    // Symbols
    builder.add_command("mod", define_symbol("mod"));
    builder.add_command("cdot", define_symbol("dot.c"));
    builder.add_command("times", define_symbol("times"));
    builder.add_command("oplus", define_symbol("plus.circle"));
    builder.add_command("ominus", define_symbol("minus.circle"));
    builder.add_command("pm", define_symbol("plus.minus"));
    builder.add_command("mp", define_symbol("minus.plus"));
    builder.add_command("div", define_symbol("div"));
    builder.add_command("star", define_symbol("star"));
    builder.add_command("cap", define_symbol("sect"));
    builder.add_command("cup", define_symbol("union"));
    builder.add_command("in", define_symbol("in"));
    builder.add_command("notin", define_symbol("in.not"));
    builder.add_command("subset", define_symbol("subset"));
    builder.add_command("subseteq", define_symbol("subset.eq"));
    builder.add_command("neq", define_symbol("!="));
    builder.add_command("lt", define_symbol("<"));
    builder.add_command("gt", define_symbol(">"));
    builder.add_command("le", define_symbol("<="));
    builder.add_command("ge", define_symbol(">="));
    builder.add_command("leq", define_symbol("<="));
    builder.add_command("geq", define_symbol(">="));
    builder.add_command("leqslant", define_symbol("lt.eq.slant"));
    builder.add_command("geqslant", define_symbol("gt.eq.slant"));
    builder.add_command("approx", define_symbol("approx"));
    builder.add_command("leftarrow", define_symbol("<-"));
    builder.add_command("rightarrow", define_symbol("->"));
    builder.add_command("leftrightarrow", define_symbol("<->"));
    builder.add_command("Leftarrow", define_symbol("arrow.l.double"));
    builder.add_command("Rightarrow", define_symbol("=>"));
    builder.add_command("Leftrightarrow", define_symbol("<=>"));
    builder.add_command("larr", define_symbol("<-"));
    builder.add_command("rarr", define_symbol("->"));
    builder.add_command("lrarr", define_symbol("<->"));
    builder.add_command("lArr", define_symbol("arrow.l.double"));
    builder.add_command("rArr", define_symbol("=>"));
    builder.add_command("lrArr", define_symbol("<=>"));
    builder.add_command("Larr", define_symbol("arrow.l.double"));
    builder.add_command("Rarr", define_symbol("=>"));
    builder.add_command("Lrarr", define_symbol("<=>"));
    builder.add_command("longleftarrow", define_symbol("<--"));
    builder.add_command("longrightarrow", define_symbol("-->"));
    builder.add_command("longleftrightarrow", define_symbol("<-->"));
    builder.add_command("Longleftarrow", define_symbol("<=="));
    builder.add_command("Longrightarrow", define_symbol("==>"));
    builder.add_command("Longleftrightarrow", define_symbol("<==>"));
    builder.add_command("to", define_symbol("->"));
    builder.add_command("mapsto", define_symbol("|->"));
    builder.add_command("infty", define_symbol("oo"));
    builder.add_command("lbrack", define_symbol("bracket.l"));
    builder.add_command("rbrack", define_symbol("bracket.r"));
    builder.add_command("angle", define_symbol("angle"));
    builder.add_command("langle", define_symbol("angle.l"));
    builder.add_command("rangle", define_symbol("angle.r"));
    builder.add_command("measuredangle", define_symbol("angle.arc"));
    builder.add_command("sphericalangle", define_symbol("angle.spheric"));
    builder.add_command("ast", define_symbol("ast"));
    builder.add_command("circledast", define_symbol("ast.circle"));
    builder.add_command("backslash", define_symbol("backslash"));
    builder.add_command("dagger", define_symbol("dagger"));
    builder.add_command("ddagger", define_symbol("dagger.double"));
    builder.add_command("circleddash", define_symbol("dash.circle"));
    builder.add_command("odot", define_symbol("dot.circle"));
    builder.add_command("bigodot", define_symbol("dot.circle.big"));
    builder.add_command("boxdot", define_symbol("dot.square"));
    builder.add_command("cdots", define_symbol("dots.h.c"));
    builder.add_command("ldots", define_symbol("dots.h"));
    builder.add_command("vdots", define_symbol("dots.v"));
    builder.add_command("ddots", define_symbol("dots.down"));
    builder.add_command("sim", define_symbol("tilde"));
    builder.add_command("simeq", define_symbol("tilde.eq"));
    builder.add_command("backsimeq", define_symbol("tilde.eq.rev"));
    builder.add_command("cong", define_symbol("tilde.equiv"));
    builder.add_command("ncong", define_symbol("tilde.equiv.not"));
    builder.add_command("nsim", define_symbol("tilde.not"));
    builder.add_command("backsim", define_symbol("tilde.rev"));
    builder.add_command("prime", define_symbol("prime"));
    builder.add_command("backprime", define_symbol("prime.rev"));
    builder.add_command("bigoplus", define_symbol("plus.circle.big"));
    builder.add_command("dotplus", define_symbol("plus.dot"));
    builder.add_command("boxplus", define_symbol("plus.square"));
    builder.add_command("boxminus", define_symbol("minus.square"));
    builder.add_command("eqsim", define_symbol("minus.tilde"));
    builder.add_command("otimes", define_symbol("times.circle"));
    builder.add_command("bigotimes", define_symbol("times.circle.big"));
    builder.add_command("divideontimes", define_symbol("times.div"));
    builder.add_command("leftthreetimes", define_symbol("times.three.l"));
    builder.add_command("rightthreetimes", define_symbol("times.three.r"));
    builder.add_command("ltimes", define_symbol("times.l"));
    builder.add_command("rtimes", define_symbol("times.r"));
    builder.add_command("boxtimes", define_symbol("times.square"));
    builder.add_command("triangleq", define_symbol("eq.delta"));
    builder.add_command("curlyeqprec", define_symbol("eq.prec"));
    builder.add_command("curlyeqsucc", define_symbol("eq.succ"));
    builder.add_command("gtrdot", define_symbol("gt.dot"));
    builder.add_command("gg", define_symbol("gt.double"));
    builder.add_command("gtreqless", define_symbol("gt.eq.lt"));
    builder.add_command("ngeq", define_symbol("gt.eq.not"));
    builder.add_command("geqq", define_symbol("gt.equiv"));
    builder.add_command("gtrless", define_symbol("gt.lt"));
    builder.add_command("gneqq", define_symbol("gt.nequiv"));
    builder.add_command("ngtr", define_symbol("gt.not"));
    builder.add_command("gnsim", define_symbol("gt.ntilde"));
    builder.add_command("gtrsim", define_symbol("gt.tilde"));
    builder.add_command("vartriangleright", define_symbol("gt.tri"));
    builder.add_command("trianglerighteq", define_symbol("gt.tri.eq"));
    builder.add_command("ntrianglerighteq", define_symbol("gt.tri.eq.not"));
    builder.add_command("ntriangleright", define_symbol("gt.tri.not"));
    builder.add_command("ggg", define_symbol("gt.triple"));
    builder.add_command("lessdot", define_symbol("lt.dot"));
    builder.add_command("ll", define_symbol("lt.double"));
    builder.add_command("lesseqgtr", define_symbol("lt.eq.gt"));
    builder.add_command("nleq", define_symbol("lt.eq.not"));
    builder.add_command("leqq", define_symbol("lt.equiv"));
    builder.add_command("lessgtr", define_symbol("lt.gt"));
    builder.add_command("lneqq", define_symbol("lt.nequiv"));
    builder.add_command("nless", define_symbol("lt.not"));
    builder.add_command("lnsim", define_symbol("lt.ntilde"));
    builder.add_command("lesssim", define_symbol("lt.tilde"));
    builder.add_command("vartriangleleft", define_symbol("lt.tri"));
    builder.add_command("trianglelefteq", define_symbol("lt.tri.eq"));
    builder.add_command("ntrianglelefteq", define_symbol("lt.tri.eq.not"));
    builder.add_command("ntriangleleft", define_symbol("lt.tri.not"));
    builder.add_command("lll", define_symbol("lt.triple"));
    builder.add_command("approxeq", define_symbol("approx.eq"));
    builder.add_command("prec", define_symbol("prec"));
    builder.add_command("precapprox", define_symbol("prec.approx"));
    builder.add_command("preccurlyeq", define_symbol("prec.eq"));
    builder.add_command("npreceq", define_symbol("prec.eq.not"));
    builder.add_command("precnapprox", define_symbol("prec.napprox"));
    builder.add_command("nprec", define_symbol("prec.not"));
    builder.add_command("precnsim", define_symbol("prec.ntilde"));
    builder.add_command("precsim", define_symbol("prec.tilde"));
    builder.add_command("succ", define_symbol("succ"));
    builder.add_command("succapprox", define_symbol("succ.approx"));
    builder.add_command("succcurlyeq", define_symbol("succ.eq"));
    builder.add_command("nsucceq", define_symbol("succ.eq.not"));
    builder.add_command("succnapprox", define_symbol("succ.napprox"));
    builder.add_command("nsucc", define_symbol("succ.not"));
    builder.add_command("succnsim", define_symbol("succ.ntilde"));
    builder.add_command("succsim", define_symbol("succ.tilde"));
    builder.add_command("equiv", define_symbol("equiv"));
    builder.add_command("propto", define_symbol("prop"));
    builder.add_command("varnothing", define_symbol("nothing"));
    builder.add_command("smallsetminus", define_symbol("without"));
    builder.add_command("complement", define_symbol("complement"));
    builder.add_command("ni", define_symbol("in.rev"));
    builder.add_command("Subset", define_symbol("subset.double"));
    builder.add_command("nsubseteq", define_symbol("subset.eq.not"));
    builder.add_command("sqsubseteq", define_symbol("subset.eq.sq"));
    builder.add_command("subsetneq", define_symbol("subset.neq"));
    builder.add_command("supset", define_symbol("supset"));
    builder.add_command("Supset", define_symbol("supset.double"));
    builder.add_command("supseteq", define_symbol("supset.eq"));
    builder.add_command("nsupseteq", define_symbol("supset.eq.not"));
    builder.add_command("sqsupseteq", define_symbol("supset.eq.sq"));
    builder.add_command("supsetneq", define_symbol("supset.neq"));
    builder.add_command("bigcup", define_symbol("union.big"));
    builder.add_command("Cup", define_symbol("union.double"));
    builder.add_command("uplus", define_symbol("union.plus"));
    builder.add_command("biguplus", define_symbol("union.plus.big"));
    builder.add_command("sqcup", define_symbol("union.sq"));
    builder.add_command("bigsqcup", define_symbol("union.sq.big"));
    builder.add_command("bigcap", define_symbol("sect.big"));
    builder.add_command("Cap", define_symbol("sect.double"));
    builder.add_command("sqcap", define_symbol("sect.sq"));
    builder.add_command("partial", define_symbol("diff"));
    builder.add_command("nabla", define_symbol("nabla"));
    builder.add_command("coprod", define_symbol("product.co"));
    builder.add_command("forall", define_symbol("forall"));
    builder.add_command("exists", define_symbol("exists"));
    builder.add_command("nexists", define_symbol("exists.not"));
    builder.add_command("top", define_symbol("top"));
    builder.add_command("bot", define_symbol("bot"));
    builder.add_command("neg", define_symbol("not"));
    builder.add_command("land", define_symbol("and"));
    builder.add_command("bigwedge", define_symbol("and.big"));
    builder.add_command("curlywedge", define_symbol("and.curly"));
    builder.add_command("vee", define_symbol("or"));
    builder.add_command("bigvee", define_symbol("or.big"));
    builder.add_command("curlyvee", define_symbol("or.curly"));
    builder.add_command("models", define_symbol("models"));
    builder.add_command("therefore", define_symbol("therefore"));
    builder.add_command("because", define_symbol("because"));
    builder.add_command("blacksquare", define_symbol("qed"));
    builder.add_command("circ", define_symbol("compose"));
    builder.add_command("multimap", define_symbol("multimap"));
    builder.add_command("mid", define_symbol("divides"));
    builder.add_command("nmid", define_symbol("divides.not"));
    builder.add_command("wr", define_symbol("wreath"));
    builder.add_command("parallel", define_symbol("parallel"));
    builder.add_command("nparallel", define_symbol("parallel.not"));
    builder.add_command("perp", define_symbol("perp"));
    builder.add_command("Join", define_symbol("join"));
    builder.add_command("pounds", define_symbol("pound"));
    builder.add_command("clubsuit", define_symbol("suit.club"));
    builder.add_command("spadesuit", define_symbol("suit.spade"));
    builder.add_command("bullet", define_symbol("bullet"));
    builder.add_command("circledcirc", define_symbol("circle.nested"));
    builder.add_command("rhd", define_symbol("triangle.stroked.r"));
    builder.add_command("lhd", define_symbol("triangle.stroked.l"));
    builder.add_command("bigtriangleup", define_symbol("triangle.stroked.t"));
    builder.add_command("bigtriangledown", define_symbol("triangle.stroked.b"));
    builder.add_command("triangleright", define_symbol("triangle.stroked.small.r"));
    builder.add_command("triangledown", define_symbol("triangle.stroked.small.b"));
    builder.add_command("triangleleft", define_symbol("triangle.stroked.small.l"));
    builder.add_command("vartriangle", define_symbol("triangle.stroked.small.t"));
    builder.add_command("square", define_symbol("square.stroked"));
    builder.add_command("Diamond", define_symbol("diamond.stroked"));
    builder.add_command("diamond", define_symbol("diamond.stroked.small"));
    builder.add_command("lozenge", define_symbol("lozenge.stroked"));
    builder.add_command("blacklozenge", define_symbol("lozenge.filled"));
    builder.add_command("bigstar", define_symbol("star.stroked"));
    builder.add_command("longmapsto", define_symbol("arrow.r.long.bar"));
    builder.add_command("nRightarrow", define_symbol("arrow.r.double.not"));
    builder.add_command("hookrightarrow", define_symbol("arrow.r.hook"));
    builder.add_command("looparrowright", define_symbol("arrow.r.loop"));
    builder.add_command("nrightarrow", define_symbol("arrow.r.not"));
    builder.add_command("rightsquigarrow", define_symbol("arrow.r.squiggly"));
    builder.add_command("rightarrowtail", define_symbol("arrow.r.tail"));
    builder.add_command("Rrightarrow", define_symbol("arrow.r.triple"));
    builder.add_command("twoheadrightarrow", define_symbol("arrow.r.twohead"));
    builder.add_command("nLeftarrow", define_symbol("arrow.l.double.not"));
    builder.add_command("hookleftarrow", define_symbol("arrow.l.hook"));
    builder.add_command("looparrowleft", define_symbol("arrow.l.loop"));
    builder.add_command("nleftarrow", define_symbol("arrow.l.not"));
    builder.add_command("leftarrowtail", define_symbol("arrow.l.tail"));
    builder.add_command("Lleftarrow", define_symbol("arrow.l.triple"));
    builder.add_command("twoheadleftarrow", define_symbol("arrow.l.twohead"));
    builder.add_command("uparrow", define_symbol("arrow.t"));
    builder.add_command("Uparrow", define_symbol("arrow.t.double"));
    builder.add_command("downarrow", define_symbol("arrow.b"));
    builder.add_command("Downarrow", define_symbol("arrow.b.double"));
    builder.add_command("nLeftrightarrow", define_symbol("arrow.l.r.double.not"));
    builder.add_command("nleftrightarrow", define_symbol("arrow.l.r.not"));
    builder.add_command("leftrightsquigarrow", define_symbol("arrow.l.r.wave"));
    builder.add_command("updownarrow", define_symbol("arrow.t.b"));
    builder.add_command("Updownarrow", define_symbol("arrow.t.b.double"));
    builder.add_command("nearrow", define_symbol("arrow.tr"));
    builder.add_command("searrow", define_symbol("arrow.br"));
    builder.add_command("nwarrow", define_symbol("arrow.tl"));
    builder.add_command("swarrow", define_symbol("arrow.bl"));
    builder.add_command("circlearrowleft", define_symbol("arrow.ccw"));
    builder.add_command("curvearrowleft", define_symbol("arrow.ccw.half"));
    builder.add_command("circlearrowright", define_symbol("arrow.cw"));
    builder.add_command("curvearrowright", define_symbol("arrow.cw.half"));
    builder.add_command("rightrightarrows", define_symbol("arrows.rr"));
    builder.add_command("leftleftarrows", define_symbol("arrows.ll"));
    builder.add_command("upuparrows", define_symbol("arrows.tt"));
    builder.add_command("downdownarrows", define_symbol("arrows.bb"));
    builder.add_command("leftrightarrows", define_symbol("arrows.lr"));
    builder.add_command("rightleftarrows", define_symbol("arrows.rl"));
    builder.add_command("rightharpoonup", define_symbol("harpoon.rt"));
    builder.add_command("rightharpoondown", define_symbol("harpoon.rb"));
    builder.add_command("leftharpoonup", define_symbol("harpoon.lt"));
    builder.add_command("leftharpoondown", define_symbol("harpoon.lb"));
    builder.add_command("upharpoonleft", define_symbol("harpoon.tl"));
    builder.add_command("upharpoonright", define_symbol("harpoon.tr"));
    builder.add_command("downharpoonleft", define_symbol("harpoon.bl"));
    builder.add_command("downharpoonright", define_symbol("harpoon.br"));
    builder.add_command("leftrightharpoons", define_symbol("harpoons.ltrb"));
    builder.add_command("rightleftharpoons", define_symbol("harpoons.rtlb"));
    builder.add_command("vdash", define_symbol("tack.r"));
    builder.add_command("nvdash", define_symbol("tack.r.not"));
    builder.add_command("vDash", define_symbol("tack.r.double"));
    builder.add_command("nvDash", define_symbol("tack.r.double.not"));
    builder.add_command("dashv", define_symbol("tack.l"));
    builder.add_command("hbar", define_symbol("planck.reduce"));
    builder.add_command("hslash", define_symbol("planck.reduce"));
    builder.add_command("Re", define_symbol("Re"));
    builder.add_command("Im", define_symbol("Im"));
    builder.add_command("imath", define_symbol("dotless.i"));
    builder.add_command("jmath", define_symbol("dotless.j"));
    builder.add_command("lbrace", define_symbol("\\{"));
    builder.add_command("rbrace", define_symbol("\\}"));
    // Matrices
    builder.add_command("matrix", TEX_MATRIX_ENV);
    builder.add_command("pmatrix", TEX_MATRIX_ENV);
    builder.add_command("bmatrix", TEX_MATRIX_ENV);
    builder.add_command("Bmatrix", TEX_MATRIX_ENV);
    builder.add_command("vmatrix", TEX_MATRIX_ENV);
    builder.add_command("Vmatrix", TEX_MATRIX_ENV);
    builder.add_command("array", define_matrix_env(Some(1), "mitexarray"));
    // Environments
    builder.add_command("aligned", TEX_NORMAL_ENV);
    builder.add_command("align", define_normal_env(None, "aligned"));
    builder.add_command("align*", define_normal_env(None, "aligned"));
    builder.add_command("equation", define_normal_env(None, "aligned"));
    builder.add_command("equation*", define_normal_env(None, "aligned"));
    builder.add_command("split", define_normal_env(None, "aligned"));
    builder.add_command("gather", define_normal_env(None, "aligned"));
    builder.add_command("cases", CommandSpecItem::Env(EnvShape {
        args: ArgPattern::None,
        ctx_feature: ContextFeature::IsCases,
        alias: Some("cases".to_owned()),
    }));
    // Specials
    builder.add_command("label", define_command_with_alias(1, "mitexlabel"));
    builder.add_command("vspace", TEX_CMD1);
    builder.add_command("hspace", TEX_CMD1);
    builder.add_command("text", TEX_CMD1);
    builder.add_command(
        "over",
        CommandSpecItem::Cmd(CmdShape {
            args: ArgShape::InfixGreedy,
            alias: Some("frac".to_owned()),
        }),
    );

    builder.add_command("sqrt", define_glob_command("{,b}t", "mitexsqrt"));
    builder.build()
}

static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(default_spec);

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

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
            "zws zws integral _(1 )^(2 ) x  upright(d  )x ",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$\underline{T}$"#), @r###"
        Ok(
            "underline(T )",
        )
        "###);
    }

    #[test]
    fn test_convert_frac() {
        assert_debug_snapshot!(convert_math(r#"$\frac{a}{b}$"#), @r###"
        Ok(
            "frac(a ,b )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$\frac 12_3$"#), @r###"
        Ok(
            "zws frac( 1 ,2 )_(3 )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_displaystyle() {
        assert_debug_snapshot!(convert_math(r#"$\displaystyle xyz\frac{1}{2}$"#), @r###"
        Ok(
            "mitexdisplay( x y z ,frac(1 ,2 ))",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$1 + {\displaystyle 23} + 4$"#), @r###"
        Ok(
            "1  +  mitexdisplay( 2 3 ) +  4 ",
        )
        "###
        );
    }

    #[test]
    fn test_convert_limits() {
        assert_debug_snapshot!(convert_math(r#"\sum\limits_1^2$"#), @r###"
        Ok(
            "zws zws limits(sum )_(1 )^(2 )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_subsup() {
        assert_debug_snapshot!(convert_math(r#"x_1^2$"#), @r###"
        Ok(
            "zws zws x _(1 )^(2 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"x^2_1$"#), @r###"
        Ok(
            "zws zws x ^(2 )_(1 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"x''_1$"#), @r###"
        Ok(
            "zws zws zws x ' ' _(1 )",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"x_1''$"#), @r###"
        Ok(
            "zws zws zws x _(1 )' ' ",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"{}_1^2x_3^4$"#), @r###"
        Ok(
            "zws zws _(1 )^(2 )zws zws x _(3 )^(4 )",
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
            "1  +  frac(2  , 3 )",
        )
        "###
        );
        // assert_debug_snapshot!(convert_math(r#"${l \over 2'}$"#), @r###"
        // Ok(
        //     "a b c ",
        // )
        // "###);
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
            "x med negativespace  thin  med  med  thick  thick  quad  wide  y ",
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
            "x mitexcolor(r e d ,y z ,frac(1 ,2 ))",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$x\textcolor{red}yz$"#), @r###"
        Ok(
            "x textcolor(r e d ,y )z ",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$x\textcolor{red}{yz}$"#), @r###"
        Ok(
            "x textcolor(r e d ,y z )",
        )
        "###);
        assert_debug_snapshot!(convert_math(r#"$x\colorbox{red}yz$"#), @r###"
        Ok(
            "x colorbox(r e d ,y )z ",
        )
        "###
        );
        assert_debug_snapshot!(convert_math(r#"$x\colorbox{red}{yz}$"#), @r###"
        Ok(
            "x colorbox(r e d ,y z )",
        )
        "###
        );
    }

    #[test]
    fn test_convert_matrix() {
        assert_debug_snapshot!(convert_math(
                r#"$\begin{matrix}
        1 & 2 & 3\\
a & b & c
\end{matrix}$"#
            ),
            @r###"
        Ok(
            "matrix(1  , 2  , 3 ;\na  , b  , c \n)",
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
            "Vmatrix(1  , 2  , 3 ;\na  , b  , c \n)",
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
            "mitexarray(arg0: l c r \n        ,1  , 2  , 3 ;\na  , b  , c \n)",
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
            "aligned(1  & 2  & 3 \\\na  & b  & c \n)",
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
            "aligned(1  & 2  & 3 \\\na  & b  & c \n)",
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
    fn test_convert_text() {
        assert_debug_snapshot!(convert_math(r#"$\text{abc}$"#).unwrap(), @r###""#[abc]""###);
        assert_debug_snapshot!(convert_math(r#"$\text{ a b c }$"#).unwrap(), @r###""#[ a b c ]""###);
        assert_debug_snapshot!(convert_math(r#"$\text{abc{}}$"#).unwrap(), @r###""#[abc]""###);
        assert_debug_snapshot!(convert_math(r#"$\text{ab{}c}$"#).unwrap(), @r###""#[abc]""###);
        assert_debug_snapshot!(convert_math(r#"$\text{ab c}$"#).unwrap(), @r###""#[ab c]""###);
        // note: hack doesn't work in this case
        assert_debug_snapshot!(convert_math(r#"$\text{ab\color{red}c}$"#).unwrap(), @r###""#[ab\\colorredc]""###);
    }
}
