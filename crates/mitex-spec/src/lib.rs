//! Specification structure of a set of LaTeX commands
//!
//! The specification object will be passed to MiTeX Parser and MiTeX Converter.
//! It is used by the parser to produce ASTs which respects shape of commands.
//! It is used by the converter to convert ASTs into a valid typst code.
//!
//! Note: since we need process environments statically, users cannot override
//! `\begin`, `\end`, `\left`, and `\right` command.
//!
//! See <https://github.com/OrangeX4/mitex/blob/main/docs/spec.typ> for detailed description.

use std::{collections::HashMap, sync::Arc};

pub mod preludes;

/// An item of command specification.
/// It is either a command or an environment.
#[derive(Debug, Clone)]
pub enum CommandSpecItem {
    Cmd(CmdShape),
    Env(EnvShape),
}

/// Command specification contains a set of commands
/// and environments.
#[derive(Debug, Clone)]
pub struct CommandSpecRepr {
    /// A map from command name to command specification
    pub commands: HashMap<String, CommandSpecItem>,
}

/// Command specification that is cheap to clone
#[derive(Debug, Clone)]
pub struct CommandSpec(Arc<CommandSpecRepr>);

impl CommandSpec {
    /// Create a new command specification
    pub fn new(commands: HashMap<String, CommandSpecItem>) -> Self {
        Self(Arc::new(CommandSpecRepr { commands }))
    }

    /// Get the command specification
    pub fn get(&self, name: &str) -> Option<&CommandSpecItem> {
        self.0.commands.get(name)
    }

    /// Get the command specification in kind of command
    pub fn get_cmd(&self, name: &str) -> Option<&CmdShape> {
        self.get(name).and_then(|item| {
            if let CommandSpecItem::Cmd(item) = item {
                Some(item)
            } else {
                None
            }
        })
    }

    /// Get the command specification in kind of environment
    pub fn get_env(&self, name: &str) -> Option<&EnvShape> {
        self.get(name).and_then(|item| {
            if let CommandSpecItem::Env(item) = item {
                Some(item)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct CmdShape {
    /// Describing how could we matches the arguments of a command item
    pub args: ArgShape,
    /// Alias command for typst handler
    /// For exmaple, alias `\prod` to typst's `product`
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnvShape {
    /// Describing how could we matches the arguments of an environment item
    pub args: ArgPattern,
    /// Specify how could we process items before passing them
    /// to the typst handler
    pub ctx_feature: ContextFeature,
    /// Alias command for typst handler
    /// For exmaple, alias `pmatrix` to `pmat`
    /// And specify `let pmat = math.mat.with(delim: "(")`
    /// in scope
    pub alias: Option<String>,
}

/// The character encoding used for argument matching
pub mod argument_kind {
    /// The character used for matching argument in a term (curly group or
    /// others)
    pub const ARGUMENT_KIND_TERM: char = 't';
    /// The character used for matching argument in a bracket group
    pub const ARGUMENT_KIND_BRACKET: char = 'b';
    /// The character used for matching argument in a parenthesis group
    pub const ARGUMENT_KIND_PAREN: char = 'p';
}

/// An efficient pattern used for matching.
/// It is essential regex things but one
/// can specify the pattern by fixed, range,
/// or greedy length to achieve higher performance.
///
/// Let us show usage of glob pattern by \sqrt, which is `{,b}t`
/// Exp 1. For `\sqrt{2}{3}`, parser
///   requires the pattern to match with `tt`,
///   Here, `{,b}t` matches and
///   yields string `t` (correspond to `{2}`)
/// Exp 2. For `\sqrt[1]{2}{2}`, parser
///   requires the pattern to match with `btt`,
///   Here, `{,b}t` matches and
///   yields string `bt` (correspond to `[1]{2}`)
///
/// Kind of item to match, also see [`argument_kind`]:
/// - Bracket/b: []
/// - Parenthesis/p: ()
/// - Term/t: any rest of terms, typically {} or single char
#[derive(Debug, Clone)]
pub enum ArgPattern {
    /// None of arguments is passed, i.e. it is processed as a
    /// variable in typst.
    /// Note: this is different from FixedLenTerm(0)
    /// Where, \alpha is None, but not FixedLenTerm(0)
    /// E.g. \alpha => $alpha$
    None,
    /// Fixed length pattern, equivalent to `/t{x}/g`
    /// E.g. \hat x y => $hat(x) y$,
    /// E.g. 1 \sum\limits => $1 limits(sum)$,
    FixedLenTerm(u8),
    /// Range length pattern (as much as possible),
    /// equivalent to `/t{x,y}/g`
    /// No example
    RangeLenTerm(u8, u8),
    /// Receive terms as much as possible,
    /// equivalent to `/t*/g`
    /// E.g. \over, \displaystyle
    Greedy,
    /// Most powerful pattern, but slightly slow
    /// Note that the glob must accept all prefix of the input
    ///
    /// E.g. \sqrt has a glob pattern of `{,b}t`
    /// Description:
    /// - {,b}: first, it matches an bracket option, e.g. `\sqrt[3]`
    /// - t: it later matches a single term, e.g. `\sqrt[3]{a}` or `\sqrt{a}`
    /// Note: any prefix of the glob is valid in parse stage hence you need to
    /// check whether it is complete in later stage.
    Glob(Arc<str>),
}

// struct ArgShape(ArgPattern, Direction);

/// Shape of arguments
/// With direction to match since
/// Note: We currently only support
/// - `Direction::Right` with any `ArgPattern`
/// - `Direction::Left` with `ArgPattern::FixedLenTerm(1)`
/// - `Direction::Infix` with `ArgPattern::Greedy`
#[derive(Debug, Clone)]
pub enum ArgShape {
    /// A command that assosicates with right side of items.
    /// E.g. \hat
    Right(ArgPattern),
    /// A command that assosicates with left side of items, and with
    /// `ArgPattern::FixedLenTerm(1)`. E.g. \limits
    Left1,
    /// A command that assosicates with both side of items, and with
    /// `ArgPattern::Greedy`. Also known as infix operators.
    /// E.g. \over
    InfixGreedy,
}

#[derive(Debug, Clone)]
pub enum ContextFeature {
    /// No special feature
    None,
    /// Parse content like mat arguments
    IsMatrix,
    /// Parse content like cases
    IsCases,
}
