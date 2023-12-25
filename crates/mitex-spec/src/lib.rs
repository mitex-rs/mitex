//! Specification structure of a set of LaTeX commands.
//!
//! The specification will be passed to MiTeX for converting LaTeX code
//! correctly. For example, MiTeX Parser uses it to produce an AST that respect
//! the shape of commands.
//!
//! Note: since we need to process environments statically, users cannot
//! override the `\begin` and `\end` commands.
//!
//! See <https://github.com/mitex-rs/mitex/blob/main/docs/spec.typ> for detailed description.

use std::sync::Arc;

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

pub mod preludes;
pub mod query;
mod stream;
pub use query::CommandSpecRepr as JsonCommandSpec;

/// An item of command specification. It is either a normal _command_ or an
/// _environment_.
/// See [Command Syntax] for concept of _command_.
/// See [Environment Syntax] for concept of _environment_.
///
/// [Command Syntax]: https://latexref.xyz/LaTeX-command-syntax.html
/// [Environment Syntax]: https://latexref.xyz/Environment-syntax.html
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum CommandSpecItem {
    Cmd(CmdShape),
    Env(EnvShape),
}

/// Command specification that contains a set of commands and environments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct CommandSpecRepr {
    /// A map from command name to command specification
    pub commands: fxhash::FxHashMap<String, CommandSpecItem>,
}

/// Command specification that is cheap to clone
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct CommandSpec(Arc<CommandSpecRepr>);

#[cfg(feature = "rkyv")]
impl CommandSpec {
    pub fn to_bytes(&self) -> Vec<u8> {
        // Or you can customize your serialization for better performance
        // and compatibility with #![no_std] environments
        use rkyv::ser::{serializers::AllocSerializer, Serializer};

        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(self.0.as_ref()).unwrap();
        let bytes = serializer.into_serializer().into_inner();

        bytes.into_vec()
    }

    #[cfg(feature = "rkyv-validation")]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let s = stream::BytesModuleStream::from_slice(bytes);

        Self(Arc::new(s.checkout_owned()))
    }

    /// # Safety
    /// The data source must be trusted and valid.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        let s = stream::BytesModuleStream::from_slice(bytes);

        Self(Arc::new(s.checkout_owned_unchecked()))
    }
}

impl CommandSpec {
    /// Create a new command specification
    pub fn new(commands: fxhash::FxHashMap<String, CommandSpecItem>) -> Self {
        Self(Arc::new(CommandSpecRepr { commands }))
    }

    /// Get an item by name
    pub fn get(&self, name: &str) -> Option<&CommandSpecItem> {
        self.0.commands.get(name)
    }

    /// Get an item by name in kind of _command_
    pub fn get_cmd(&self, name: &str) -> Option<&CmdShape> {
        self.get(name).and_then(|item| {
            if let CommandSpecItem::Cmd(item) = item {
                Some(item)
            } else {
                None
            }
        })
    }

    /// Get an item by name in kind of _environment_
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
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct CmdShape {
    /// Describes how we could match the arguments of a command item.
    pub args: ArgShape,
    /// Makes the command alias to some Typst handler.
    /// For exmaple, alias `\prod` to Typst's `product`
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct EnvShape {
    /// Describes how we could match the arguments of an environment item.
    pub args: ArgPattern,
    /// Specifies how we could process items before passing them
    /// to the Typst handler
    pub ctx_feature: ContextFeature,
    /// Makes the command alias to some Typst handler.
    /// For exmaple, alias `\pmatrix` to a Typst function `pmat` in scope.
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

/// An efficient pattern used for argument matching.
///
/// There are four kinds of pattern. The most powerful one is
/// [`ArgPattern::Glob`], which matches an sequence of input as arguments. Among
/// these four kinds, [`ArgPattern::Glob`] can already match all possible inputs
/// in our use cases. But one should specify a fixed length pattern
/// ([`ArgPattern::FixedLenTerm`]), a range length pattern
/// ([`ArgPattern::RangeLenTerm`]), or a greedy pattern
/// ([`ArgPattern::Greedy`]) to achieve better performance.
///
/// Let us look at usage of a glob pattern by \sqrt, which is `{,b}t`.
///
/// - Example 1. For `\sqrt{2}{3}`, parser requires the pattern to match with an
///   encoded string `tt`. Here, `{,b}t` matches and yields the string `t`
///   (which corresponds to `{2}`).
///
/// - Example 2. For `\sqrt[1]{2}{2}`, parser requires the pattern to match with
///   an encoded string `btt`. Here, `{,b}t` matches and yields the string `bt`
///   (which corresponds to `[1]{2}`).
///
/// Kinds of item to match:
/// - Bracket/b: []
/// - Parenthesis/p: ()
/// - Term/t: any remaining terms, typically {} or a single char
///
/// Note: any prefix of the argument pattern are matched during the parse stage,
/// so you need to check whether it is complete in later stages.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ArgPattern {
    /// No arguments are passed, i.e. this is processed as a variable in Typst.
    ///
    /// E.g. `\alpha` => `$alpha$`, where `\alpha` has an argument pattern of
    /// `None`
    None,
    /// Fixed length pattern, equivalent to repeat `{,t}` for `x` times
    ///
    /// E.g. `\hat x y` => `$hat(x) y$`, where `\hat` has an argument pattern of
    /// `FixedLenTerm(1)`
    ///
    /// E.g. `1 \sum\limits` => `$1 limits(sum)$`, where `\limits` has an
    /// argument pattern of `FixedLenTerm(1)`
    FixedLenTerm(u8),
    /// Range length pattern (matches as much as possible), equivalent to
    /// repeat `t` for `x` times, then repeat `{,t}` for `y` times.
    ///
    /// No example
    RangeLenTerm(u8, u8),
    /// Receives any items as much as possible, equivalent to `*`.
    ///
    /// E.g. \over, \displaystyle
    Greedy,
    /// The most powerful pattern, but slightly slow.
    /// Note that the glob must accept the whole prefix of the input.
    ///
    /// E.g. \sqrt has a glob argument pattern of `{,b}t`
    ///
    /// Description of the glob pattern:
    /// - {,b}: first, it matches a bracket option, e.g. `\sqrt[3]`
    /// - t: it then matches a single term, e.g. `\sqrt[3]{a}` or `\sqrt{a}`
    Glob(Arc<str>),
}

// struct ArgShape(ArgPattern, Direction);

/// Shape of arguments with direction to match since.
///
/// Note: We currently only support
/// - `Direction::Right` with any `ArgPattern`
/// - `Direction::Left` with `ArgPattern::FixedLenTerm(1)`
/// - `Direction::Infix` with `ArgPattern::Greedy`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ArgShape {
    /// A command that associates with the right side of items.
    ///
    /// E.g. `\hat`
    Right(ArgPattern),
    /// A command that associates with the left side of items, and with
    /// `ArgPattern::FixedLenTerm(1)`.
    ///
    /// E.g. `\limits`
    Left1,
    /// A command that associates with both side of items, and with
    /// `ArgPattern::Greedy`, also known as infix operators.
    ///
    /// E.g. `\over`
    InfixGreedy,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ContextFeature {
    /// No special feature
    None,
    /// Parse content like mat arguments
    IsMatrix,
    /// Parse content like cases
    IsCases,
}
