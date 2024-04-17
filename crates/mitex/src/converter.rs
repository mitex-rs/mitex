use core::fmt;
use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

pub use mitex_parser::spec::*;

use mitex_parser::syntax::CmdItem;
use mitex_parser::syntax::EnvItem;
use mitex_parser::syntax::FormulaItem;
use mitex_parser::syntax::SyntaxNode;
use mitex_spec_gen::DEFAULT_SPEC;
use rowan::ast::AstNode;

#[derive(Debug, Clone, Copy, Default)]
pub enum LaTeXMode {
    #[default]
    Text,
    Math,
}

#[derive(Debug, Clone, Copy, Default)]
enum LaTeXEnv {
    #[default]
    // Text mode
    None,
    Itemize,
    Enumerate,
    // Math mode
    Math,
    Matrix,
    Cases,
    SubStack,
    MathCurlyGroup,
}

pub struct Converter {
    mode: LaTeXMode,
    env: LaTeXEnv,
    // indent for itemize and enumerate
    indent: usize,
    // label for block equation
    label: Option<String>,
    // skip the space at the beginning of the line
    skip_next_space: bool,
}

impl Converter {
    fn new(mode: LaTeXMode) -> Self {
        Self {
            mode,
            env: LaTeXEnv::default(),
            indent: 0,
            label: None,
            skip_next_space: true,
        }
    }

    #[must_use]
    fn enter_mode(&mut self, context: LaTeXMode) -> LaTeXMode {
        let prev = self.mode;
        self.mode = context;
        prev
    }

    fn exit_mode(&mut self, prev: LaTeXMode) {
        self.mode = prev;
    }

    #[must_use]
    fn enter_env(&mut self, context: LaTeXEnv) -> LaTeXEnv {
        let prev = self.env;
        self.env = context;
        if matches!(self.env, LaTeXEnv::Itemize | LaTeXEnv::Enumerate) {
            self.indent += 2;
        }
        prev
    }

    fn exit_env(&mut self, prev: LaTeXEnv) {
        if matches!(self.env, LaTeXEnv::Itemize | LaTeXEnv::Enumerate) {
            self.indent -= 2;
        }
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

impl Converter {
    fn convert(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
        use LatexSyntaxKind::*;

        match elem.kind() {
            TokenWhiteSpace => {}
            _ => {
                self.skip_next_space = false;
            }
        }
        match elem.kind() {
            TokenError => Err(match elem {
                LatexSyntaxElem::Node(node) => format!("error unexpected: {:?}", node.text()),
                LatexSyntaxElem::Token(token) => format!("error unexpected: {:?}", token.text()),
            })?,
            ItemLR | ClauseArgument | ScopeRoot | ItemText | ItemBracket | ItemParen => {
                for child in elem.as_node().unwrap().children_with_tokens() {
                    self.convert(f, child, spec)?;
                }
            }
            ItemFormula => {
                self.convert_formula(f, elem, spec)?;
            }
            ItemCurly => {
                self.convert_curly_group(f, elem, spec)?;
            }
            ClauseLR => {
                self.convert_clause_lr(f, elem, spec)?;
            }
            ItemAttachComponent => {
                self.convert_attach_component(f, elem, spec)?;
            }
            ClauseCommandName => Err("command name outside of command".to_owned())?,
            ItemBegin | ItemEnd => Err("clauses outside of environment".to_owned())?,
            TokenWord => {
                if matches!(self.mode, LaTeXMode::Math) {
                    // break up words into individual characters and add a space
                    let text = elem.as_token().unwrap().text().to_string();
                    for prev in text.chars() {
                        f.write_char(prev)?;
                        f.write_char(' ')?;
                    }
                } else {
                    // write the word directly in text mode
                    f.write_str(elem.as_token().unwrap().text())?;
                }
            }
            // do nothing
            TokenLBrace | TokenRBrace | TokenDollar | TokenBeginMath | TokenEndMath
            | TokenComment | ItemBlockComment => {}
            // space identical
            TokenWhiteSpace => {
                // indent for itemize and enumerate
                if self.skip_next_space {
                    self.skip_next_space = false;
                    return Ok(());
                }
                write!(f, "{}", elem.as_token().unwrap().text())?;
            }
            TokenLineBreak => {
                write!(f, "{}", elem.as_token().unwrap().text())?;
                // indent for itemize and enumerate
                for _ in 0..self.indent {
                    f.write_char(' ')?;
                }
                self.skip_next_space = true;
            }
            // escapes
            TokenApostrophe => {
                f.write_char('\'')?;
            }
            TokenComma => {
                f.write_str("\\,")?;
            }
            TokenSlash => {
                f.write_str("\\/")?;
            }
            TokenCaret => {
                f.write_str("\\^")?;
            }
            TokenUnderscore => {
                f.write_str("\\_")?;
            }
            TokenHash => {
                f.write_str("\\#")?;
            }
            TokenAsterisk => {
                f.write_str("\\*")?;
            }
            TokenAtSign => {
                f.write_str("\\@")?;
            }
            TokenDitto => {
                f.write_str("\\\"")?;
            }
            TokenSemicolon => {
                f.write_str("\\;")?;
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
            TokenTilde => {
                if matches!(self.mode, LaTeXMode::Math) {
                    f.write_str("space.nobreak ")?;
                } else {
                    f.write_str("\\~")?;
                }
            }
            TokenAmpersand => match self.env {
                LaTeXEnv::Matrix => f.write_str("zws ,")?,
                _ => f.write_str("&")?,
            },
            ItemNewLine => match self.env {
                LaTeXEnv::Matrix => f.write_str("zws ;")?,
                LaTeXEnv::Cases => f.write_str(",")?,
                LaTeXEnv::MathCurlyGroup => {}
                _ => f.write_str("\\ ")?,
            },
            TokenCommandSym => {
                self.convert_command_sym(f, elem, spec)?;
            }
            ItemCmd => {
                let cmd = CmdItem::cast(elem.as_node().unwrap().clone()).unwrap();
                let name = cmd.name_tok().unwrap();
                let name = name.text();
                // remove prefix \
                let name = &name[1..];

                match name {
                    "item" => {
                        self.convert_command_item(f)?;
                    }
                    "label" => {
                        self.convert_command_label(f, &cmd)?;
                    }
                    _ => {
                        self.convert_normal_command(f, elem, spec)?;
                    }
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
                    ContextFeature::IsMath => LaTeXEnv::Math,
                    ContextFeature::IsMatrix => LaTeXEnv::Matrix,
                    ContextFeature::IsCases => LaTeXEnv::Cases,
                    ContextFeature::IsItemize => LaTeXEnv::Itemize,
                    ContextFeature::IsEnumerate => LaTeXEnv::Enumerate,
                };

                // hack for itemize and enumerate
                if matches!(env_kind, LaTeXEnv::Itemize | LaTeXEnv::Enumerate) {
                    let prev = self.enter_env(env_kind);

                    for child in elem.as_node().unwrap().children_with_tokens() {
                        if matches!(child.kind(), ItemBegin | ItemEnd) {
                            continue;
                        }

                        self.convert(f, child, spec)?;
                    }

                    self.exit_env(prev);

                    return Ok(());
                }

                // text mode to math mode with $ ... $
                let is_need_dollar = matches!(self.mode, LaTeXMode::Text)
                    && !matches!(
                        env_kind,
                        LaTeXEnv::None | LaTeXEnv::Itemize | LaTeXEnv::Enumerate
                    );
                let prev = self.enter_env(env_kind);
                let mut prev_mode = LaTeXMode::Text;
                if is_need_dollar {
                    f.write_str("$ ")?;
                    prev_mode = self.enter_mode(LaTeXMode::Math);
                }

                // environment name
                f.write_str(typst_name)?;
                f.write_char('(')?;
                // named args
                for (index, arg) in args.enumerate() {
                    f.write_str(format!("arg{}: ", index).as_str())?;
                    self.convert(f, rowan::NodeOrToken::Node(arg), spec)?;
                    f.write_char(',')?;
                }

                for child in elem.as_node().unwrap().children_with_tokens() {
                    if matches!(child.kind(), ItemBegin | ItemEnd) {
                        continue;
                    }

                    self.convert(f, child, spec)?;
                }

                f.write_char(')')?;

                self.exit_env(prev);

                if is_need_dollar {
                    f.write_str(" $")?;
                    self.exit_mode(prev_mode);
                }

                // handle label
                if matches!(
                    self.env,
                    LaTeXEnv::None | LaTeXEnv::Itemize | LaTeXEnv::Enumerate
                ) {
                    if let Some(label) = self.label.take() {
                        f.write_char('<')?;
                        f.write_str(label.as_str())?;
                        f.write_char('>')?;
                        self.label = None;
                    }
                }
            }
            ItemTypstCode => {
                write!(f, "{}", elem.as_node().unwrap().text())?;
            }
        };

        Ok(())
    }

    /// Convert formula like `$x$` or `$$x$$`
    fn convert_formula(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
        let formula = FormulaItem::cast(elem.as_node().unwrap().clone()).unwrap();
        if !formula.is_valid() {
            Err("formula is not valid".to_owned())?
        }
        if matches!(self.mode, LaTeXMode::Text) {
            if formula.is_inline() {
                f.write_str("#math.equation(block: false, $")?;
            } else {
                f.write_str("$ ")?;
            }
        }
        let prev = self.enter_mode(LaTeXMode::Math);
        for child in elem.as_node().unwrap().children_with_tokens() {
            self.convert(f, child, spec)?;
        }
        self.exit_mode(prev);
        if matches!(self.mode, LaTeXMode::Text) {
            if formula.is_inline() {
                f.write_str("$);")?;
            } else {
                f.write_str(" $")?;
            }
        }
        Ok(())
    }

    /// Convert curly group like `{abc}`
    fn convert_curly_group(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
        use LatexSyntaxKind::*;
        // deal with case like `\begin{pmatrix}x{\\}x\end{pmatrix}`
        let mut prev = LaTeXEnv::None;
        let mut enter_new_env = false;
        // hack for \substack{abc \\ bcd}
        if matches!(self.mode, LaTeXMode::Math) && !matches!(self.env, LaTeXEnv::SubStack) {
            prev = self.enter_env(LaTeXEnv::MathCurlyGroup);
            enter_new_env = true;
        }
        // whether to add zws for empty curly group
        let mut zws = true;
        for child in elem.as_node().unwrap().children_with_tokens() {
            match &child.kind() {
                TokenWhiteSpace | TokenLineBreak | TokenLBrace | TokenRBrace => {}
                _ => {
                    zws = false;
                }
            }
            self.convert(f, child, spec)?;
        }
        if matches!(self.mode, LaTeXMode::Math) {
            if zws {
                // deal with case like `{}_1^2x_3^4`
                f.write_str("zws ")?;
            }
            if enter_new_env {
                self.exit_env(prev);
            }
        }
        Ok(())
    }

    /// Convert \left and \right
    fn convert_clause_lr(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
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
                    LatexSyntaxKind::TokenWord if token.text() == "." => {}
                    _ => self.convert(f, rowan::NodeOrToken::Token(token), spec)?,
                },
            }
            // add space
            f.write_char(' ')?;
        }
        if name == "right" {
            f.write_char(')')?;
        }
        Ok(())
    }

    /// Convert attach component like `x_1^2`
    fn convert_attach_component(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
        if matches!(self.mode, LaTeXMode::Math) {
            // if there is already a base, if not, we need to add zws, like `_1^2`
            let mut based = false;
            // is the first non-empty element
            let mut first = true;
            for child in elem.as_node().unwrap().children_with_tokens() {
                if first {
                    let kind = child.as_token().map(|n| n.kind());
                    // the underscore _ or caret ^ is the split point
                    if matches!(
                        kind,
                        Some(LatexSyntaxKind::TokenUnderscore | LatexSyntaxKind::TokenCaret)
                    ) {
                        if !based {
                            f.write_str("zws")?;
                        }
                        write!(f, "{}(", child.as_token().unwrap().text())?;
                        first = false;
                        continue;
                    } else if !matches!(kind, Some(LatexSyntaxKind::TokenWhiteSpace)) {
                        based = true;
                    }
                }
                self.convert(f, child, spec)?;
            }
            if !first {
                f.write_char(')')?;
            }
        } else {
            // convert directly in text mode
            for child in elem.as_node().unwrap().children_with_tokens() {
                self.convert(f, child, spec)?;
            }
        }
        Ok(())
    }

    /// Convert command symbol like `\alpha`
    fn convert_command_sym(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
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
        Ok(())
    }

    /// Convert command `\item` for itemize and enumerate
    fn convert_command_item(&mut self, f: &mut fmt::Formatter<'_>) -> Result<(), ConvertError> {
        if matches!(self.env, LaTeXEnv::Itemize | LaTeXEnv::Enumerate) {
            f.write_char('\n')?;
            for _ in 0..(self.indent - 2) {
                f.write_char(' ')?;
            }
            if matches!(self.env, LaTeXEnv::Itemize) {
                f.write_str("- ")?;
            } else {
                f.write_str("+ ")?;
            }
        } else {
            Err("item command outside of itemize or enumerate".to_owned())?;
        }
        Ok(())
    }

    /// Convert command `\label`
    fn convert_command_label(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        cmd: &CmdItem,
    ) -> Result<(), ConvertError> {
        let arg = cmd
            .arguments()
            .next()
            .expect("\\label command must have one argument");
        // remove { and } then trim
        let label = arg.text().to_string();
        let label = &label[1..(label.len() - 1)];
        let label = label.trim();
        match self.env {
            LaTeXEnv::None | LaTeXEnv::Itemize | LaTeXEnv::Enumerate => {
                if matches!(self.mode, LaTeXMode::Text) {
                    f.write_char('<')?;
                    f.write_str(label)?;
                    f.write_char('>')?;
                }
            }
            _ => {
                self.label = Some(label.to_string());
            }
        }
        Ok(())
    }

    /// Convert normal command
    fn convert_normal_command(
        &mut self,
        f: &mut fmt::Formatter<'_>,
        elem: LatexSyntaxElem,
        spec: &CommandSpec,
    ) -> Result<(), ConvertError> {
        let cmd = CmdItem::cast(elem.as_node().unwrap().clone()).unwrap();
        let name = cmd.name_tok().unwrap();
        let name = name.text();
        // remove prefix \
        let name = &name[1..];
        let args = elem
            .as_node()
            .unwrap()
            .children_with_tokens()
            .filter(|node| node.kind() != LatexSyntaxKind::ClauseCommandName)
            .collect::<Vec<_>>();

        // get cmd_shape and arg_shape from spec
        let cmd_shape = spec
            .get_cmd(name)
            .ok_or_else(|| format!("unknown command: \\{}", name))?;
        let arg_shape = &cmd_shape.args;

        // typst alias name
        let mut typst_name = cmd_shape.alias.as_deref().unwrap_or(name);

        // hack for textbf and textit
        if matches!(self.mode, LaTeXMode::Text) {
            if name == "textbf" {
                typst_name = "#strong";
            } else if name == "textit" {
                typst_name = "#emph";
            }
        }

        // normal command
        write!(f, "{}", typst_name)?;

        // hack for \substack{abc \\ bcd}
        let mut prev = LaTeXEnv::None;
        if typst_name == "substack" {
            prev = self.enter_env(LaTeXEnv::SubStack);
        }

        if let ArgShape::Right(ArgPattern::None) = arg_shape {
            f.write_char(' ')?
        } else if let ArgShape::Right(ArgPattern::Greedy) = arg_shape {
            f.write_char('(')?;
            // there is only one arg in greedy
            let args = args
                .first()
                .unwrap()
                .as_node()
                .unwrap()
                .children_with_tokens()
                .collect::<Vec<_>>();
            let mut cnt = 0;
            let args_len = args.len();
            for arg in args {
                cnt += 1;
                let kind = arg.kind();
                self.convert(f, arg, spec)?;
                if matches!(kind, LatexSyntaxKind::ItemCurly) && cnt != args_len {
                    f.write_char(',')?;
                }
            }

            f.write_char(')')?;
        } else if matches!(self.mode, LaTeXMode::Math) && !typst_name.starts_with('#') {
            f.write_char('(')?;

            let mut cnt = 0;
            let args_len = args.len();
            for arg in args {
                cnt += 1;
                let kind = arg.kind();
                self.convert(f, arg, spec)?;
                if matches!(kind, LatexSyntaxKind::ClauseArgument) && cnt != args_len {
                    f.write_char(',')?;
                }
            }

            f.write_char(')')?;
        } else {
            // Text mode
            for arg in args {
                let kind = arg.kind();
                if matches!(kind, LatexSyntaxKind::ClauseArgument) {
                    f.write_char('[')?;
                    let prev_mode = self.enter_mode(LaTeXMode::Text);
                    self.convert(f, arg, spec)?;
                    self.exit_mode(prev_mode);
                    f.write_char(']')?;
                }
                f.write_char(';')?;
            }
        }

        // hack for \substack{abc \\ bcd}
        if typst_name == "substack" {
            self.exit_env(prev);
        }

        Ok(())
    }
}

struct TypstRepr {
    elem: LatexSyntaxElem,
    mode: LaTeXMode,
    spec: CommandSpec,
    error: Rc<RefCell<String>>,
}

impl fmt::Display for TypstRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ctx = Converter::new(self.mode);
        if let Err(e) = ctx.convert(f, self.elem.clone(), &self.spec) {
            self.error.borrow_mut().push_str(&e.to_string());
            return Err(fmt::Error);
        }
        Ok(())
    }
}

#[inline(always)]
pub fn convert_inner(
    input: &str,
    mode: LaTeXMode,
    spec: Option<CommandSpec>,
    do_parse: fn(input: &str, spec: CommandSpec) -> SyntaxNode,
) -> Result<String, String> {
    let node = do_parse(input, spec.unwrap_or_else(|| DEFAULT_SPEC.clone()));
    // println!("{:#?}", node);
    // println!("{:#?}", node.text());
    let mut output = String::new();
    let err = String::new();
    let err = Rc::new(RefCell::new(err));
    let repr = TypstRepr {
        elem: LatexSyntaxElem::Node(node),
        mode,
        spec: DEFAULT_SPEC.clone(),
        error: err.clone(),
    };
    core::fmt::write(&mut output, format_args!("{}", repr)).map_err(|_| err.borrow().to_owned())?;
    Ok(output)
}
