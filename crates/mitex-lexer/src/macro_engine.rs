//!
//! Good reference
//! - <https://latexref.xyz/>
//! - <https://en.wikibooks.org/wiki/Category:Book:TeX>
//! - <https://www.tug.org/utilities/plain/cseq.html>
//!
//! Commands in plan
//!
//! - \newcommand
//! - \newcommand*
//! - \renewcommand
//! - \renewcommand*
//! - \DeclareRobustCommand
//! - \DeclareRobustCommand*
//! - \DeclareTextCommand
//! - \DeclareTextCommandDefault
//! - \ProvideTextCommand
//! - \ProvideTextCommandDefault
//! - \providecommand
//! - \providecommand*
//! - \newenvironment
//! - \newenvironment*
//! - \renewenvironment
//! - \renewenvironment*
//! - \AtEndOfClass
//! - \AtEndOfPackage
//! - \AtBeginDocument
//! - \AtEndDocument
//!
//! - \@ifstar
//! - if
//! - ifdim
//! - iffalse
//! - ifnum
//! - ifodd
//! - iftrue
//! - ifx (restricted)
//!
//! - \DeclareOption
//! - \DeclareOption*
//! - \CurrentOption
//! - \ProcessOptions
//! - \ExecuteOptions
//! - \RequirePackage (only regards options)
//! - \RequirePackageWithOptions (only regards options)
//! - \documentclass (only regards options)
//! - \PassOptionsToClass
//! - \PassOptionsToPackage
//!
//! - \IfFileExists
//! - \InputIfFileExists
//! - \ProvidesFile
//!
//! - \ignorespaces
//! - \ignorespacesafterend
//!
//! - \def
//! - \gdef needs: globals: MacroMap<'a>,
//!
//! These commands will be definitely dropped or raise an error (since we are
//! not a tex engine)
//! - ifvoid
//! - ifinner
//! - ifhbox
//! - ifvbox
//! - ifhmode
//! - ifmmode
//! - ifvmode
//!
//! - CheckCommand
//! - CheckCommand*
//!
//! Commands to discuss, we may implement them in typst
//! - \newcounter, See 13 Counters
//! - \newlength, See 14 Lengths
//! - \newsavebox, See 14 Boxes
//! - \newtheorem
//! - \newfont
//!
//! - class commands, e.g. \ProvidesClass, \LoadClass, \LoadClassWithOptions

use std::{
    borrow::Cow,
    ops::{Deref, Range},
    sync::Arc,
};

use crate::{
    classify,
    snapshot_map::{self, SnapshotMap},
    BumpTokenStream, CommandName, PeekTok, StreamContext, Token,
};
use mitex_spec::CommandSpec;

pub type Checkpoint = (snapshot_map::Snapshot,);

type MacroMap<'a> = SnapshotMap<&'a str, Macro<'a>>;

#[derive(Debug)]
pub struct CmdMacro<'a> {
    pub name: String,
    pub num_args: usize,
    pub opt: Vec<PeekTok<'a>>,
    pub def: Vec<PeekTok<'a>>,
}

#[derive(Debug)]
pub struct EnvMacro<'a> {
    pub name: String,
    pub num_args: usize,
    pub opt: Vec<PeekTok<'a>>,
    pub begin_def: Vec<PeekTok<'a>>,
    pub end_def: Vec<PeekTok<'a>>,
}

#[derive(Debug, Clone)]
pub enum DeclareMacro {
    /// Command macro for NewCommand
    /// Synopsis, one of:
    ///
    /// \newcommand{\cmd}{defn}
    /// \newcommand{\cmd}[nargs]{defn}
    /// \newcommand{\cmd}[nargs][optargdefault]{defn}
    NewCommand,
    /// Command macro for NewCommandStar
    /// Synopsis, one of:
    ///
    /// \newcommand*{\cmd}{defn}
    /// \newcommand*{\cmd}[nargs]{defn}
    /// \newcommand*{\cmd}[nargs][optargdefault]{defn}
    NewCommandStar,
    /// Command macro for RenewCommand
    /// Synopsis, one of:
    ///
    /// \renewcommand{\cmd}{defn}
    /// \renewcommand{\cmd}[nargs]{defn}
    /// \renewcommand{\cmd}[nargs][optargdefault]{defn}
    RenewCommand,
    /// Command macro for RenewCommandStar
    /// Synopsis, one of:
    ///
    /// \renewcommand*{\cmd}{defn}
    /// \renewcommand*{\cmd}[nargs]{defn}
    /// \renewcommand*{\cmd}[nargs][optargdefault]{defn}
    RenewCommandStar,
    /// Command macro for ProvideCommand
    /// Synopsis, one of:
    ///
    /// \providecommand{\cmd}{defn}
    /// \providecommand{\cmd}[nargs]{defn}
    /// \providecommand{\cmd}[nargs][optargdefault]{defn}
    ProvideCommand,
    /// Command macro for ProvideCommandStar
    /// Synopsis, one of:
    ///
    /// \providecommand*{\cmd}{defn}
    /// \providecommand*{\cmd}[nargs]{defn}
    /// \providecommand*{\cmd}[nargs][optargdefault]{defn}
    ProvideCommandStar,
    /// Command macro for DeclareRobustCommand
    /// Synopsis, one of:
    ///
    /// \DeclareRobustCommand{\cmd}{defn}
    /// \DeclareRobustCommand{\cmd}[nargs]{defn}
    /// \DeclareRobustCommand{\cmd}[nargs][optargdefault]{defn}
    DeclareRobustCommand,
    /// Command macro for DeclareRobustCommandStar
    /// Synopsis, one of:
    ///
    /// \DeclareRobustCommand*{\cmd}{defn}
    /// \DeclareRobustCommand*{\cmd}[nargs]{defn}
    /// \DeclareRobustCommand*{\cmd}[nargs][optargdefault]{defn}
    DeclareRobustCommandStar,
    /// Command macro for DeclareTextCommand
    /// Synopsis, one of:
    ///
    /// \DeclareTextCommand{\cmd}{encoding}{defn}
    /// \DeclareTextCommand{\cmd}{encoding}[nargs]{defn}
    /// \DeclareTextCommand{\cmd}{encoding}[nargs][optargdefault]{defn}
    DeclareTextCommand,
    /// Command macro for DeclareTextCommandDefault
    /// Synopsis,
    /// \DeclareTextCommandDefault{\cmd}{defn}
    DeclareTextCommandDefault,
    /// Command macro for ProvideTextCommand
    /// Synopsis, one of:
    ///
    /// \ProvideTextCommand{\cmd}{encoding}{defn}
    /// \ProvideTextCommand{\cmd}{encoding}[nargs]{defn}
    /// \ProvideTextCommand{\cmd}{encoding}[nargs][optargdefault]{defn}
    ProvideTextCommand,
    /// Command macro for ProvideTextCommandDefault
    /// Synopsis,
    /// \ProvideTextCommandDefault{\cmd}{defn}
    ProvideTextCommandDefault,
    /// Command macro for NewEnvironment
    /// Synopsis, one of:
    ///
    /// \newenvironment{env}{begdef}{enddef}
    /// \newenvironment{env}[nargs]{begdef}{enddef}
    /// \newenvironment{env}[nargs][optargdefault]{begdef}{enddef}
    NewEnvironment,
    /// Command macro for NewEnvironmentStar
    /// Synopsis, one of:
    ///
    /// \newenvironment*{env}{begdef}{enddef}
    /// \newenvironment*{env}[nargs]{begdef}{enddef}
    /// \newenvironment*{env}[nargs][optargdefault]{begdef}{enddef}
    NewEnvironmentStar,
    /// Command macro for RenewEnvironment
    /// Synopsis, one of:
    ///
    /// \renewenvironment{env}{begdef}{enddef}
    /// \renewenvironment{env}[nargs]{begdef}{enddef}
    /// \renewenvironment{env}[nargs][optargdefault]{begdef}{enddef}
    RenewEnvironment,
    /// Command macro for RenewEnvironmentStar
    /// Synopsis, one of:
    ///
    /// \renewenvironment*{env}{begdef}{enddef}
    /// \renewenvironment*{env}[nargs]{begdef}{enddef}
    /// \renewenvironment*{env}[nargs][optargdefault]{begdef}{enddef}
    RenewEnvironmentStar,
    /// Command macro for AtEndOfClass
    /// Synopsis,
    /// \AtEndOfClass{code}
    AtEndOfClass,
    /// Command macro for AtEndOfPackage
    /// Synopsis,
    /// \AtEndOfPackage{code}
    AtEndOfPackage,
    /// Command macro for AtBeginDocument
    /// Synopsis,
    /// \AtBeginDocument{code}
    AtBeginDocument,
    /// Command macro for AtEndDocument
    /// Synopsis,
    /// \AtEndDocument{code}
    AtEndDocument,
}

fn define_declarative_macros(macros: &mut MacroMap) {
    for (name, value) in [
        ("newcommand", DeclareMacro::NewCommand),
        ("newcommand*", DeclareMacro::NewCommandStar),
        ("renewcommand", DeclareMacro::RenewCommand),
        ("renewcommand*", DeclareMacro::RenewCommandStar),
        ("providecommand", DeclareMacro::ProvideCommand),
        ("providecommand*", DeclareMacro::ProvideCommandStar),
        ("DeclareRobustCommand", DeclareMacro::DeclareRobustCommand),
        (
            "DeclareRobustCommand*",
            DeclareMacro::DeclareRobustCommandStar,
        ),
        ("DeclareTextCommand", DeclareMacro::DeclareTextCommand),
        (
            "DeclareTextCommandDefault",
            DeclareMacro::DeclareTextCommandDefault,
        ),
        ("ProvideTextCommand", DeclareMacro::ProvideTextCommand),
        (
            "ProvideTextCommandDefault",
            DeclareMacro::ProvideTextCommandDefault,
        ),
        ("newenvironment", DeclareMacro::NewEnvironment),
        ("newenvironment*", DeclareMacro::NewEnvironmentStar),
        ("renewenvironment", DeclareMacro::RenewEnvironment),
        ("renewenvironment*", DeclareMacro::RenewEnvironmentStar),
        ("AtEndOfClass", DeclareMacro::AtEndOfClass),
        ("AtEndOfPackage", DeclareMacro::AtEndOfPackage),
        ("AtBeginDocument", DeclareMacro::AtBeginDocument),
        ("AtEndDocument", DeclareMacro::AtEndDocument),
    ]
    .into_iter()
    {
        macros.insert(name, Macro::Declare(value));
    }
}

static DEFAULT_MACROS: once_cell::sync::Lazy<MacroMap<'static>> =
    once_cell::sync::Lazy::new(|| {
        let mut macros = MacroMap::default();
        define_declarative_macros(&mut macros);
        macros
    });

#[derive(Debug, Clone)]
pub enum Macro<'a> {
    /// Builtin macro for defining new macros
    Declare(DeclareMacro),
    /// Command macro
    Cmd(Arc<CmdMacro<'a>>),
    /// Environment macro
    Env(Arc<EnvMacro<'a>>),
}

#[derive(Debug)]
pub struct MacroState<T> {
    pub reading: Arc<T>,
    /// The real num of arguments read by engine
    pub arg_protect: u8,
    /// The cursor of tokens in macro definition
    pub has_read_tokens: u32,
}

#[derive(Debug)]
pub enum MacroNode<'a> {
    Cmd(MacroState<CmdMacro<'a>>),
    EnvBegin(MacroState<EnvMacro<'a>>),
    EnvEnd(MacroState<EnvMacro<'a>>),
    ArgSlot(Range<usize>),
    HalfReadingTok(Range<usize>),
}

/// MacroEngine has exact same interface as Lexer, but it expands macros.
///
/// When it meets a macro in token stream, It evaluates a macro into expanded
/// tokens.
pub struct MacroEngine<'a> {
    /// Command specification
    pub spec: CommandSpec,
    /// Scoped unified table of macros
    pub macros: Cow<'a, MacroMap<'a>>,
    /// Environment stack
    env_stack: Vec<EnvMacro<'a>>,
    /// Macro stack
    pub reading_macro: Vec<MacroNode<'a>>,
    /// Toekns used by macro stack
    pub scanned_tokens: Vec<PeekTok<'a>>,
}

impl<'a> BumpTokenStream<'a> for MacroEngine<'a> {
    fn bump(&mut self, ctx: &mut StreamContext<'a>) {
        self.do_bump(ctx);
    }
}

impl<'a> MacroEngine<'a> {
    /// Create a new macro engine
    pub fn new(spec: CommandSpec) -> Self {
        Self {
            spec,
            macros: std::borrow::Cow::Borrowed(DEFAULT_MACROS.deref()),
            env_stack: Vec::new(),
            reading_macro: Vec::new(),
            scanned_tokens: Vec::new(),
        }
    }

    /// fills the peek cache with a page of tokens at the same time
    fn do_bump(&mut self, ctx: &mut StreamContext<'a>) {
        /// The size of a page, in some architectures it is 16384B but that
        /// doesn't matter
        const PAGE_SIZE: usize = 4096;
        /// The item size of the peek cache
        const PEEK_CACHE_SIZE: usize = (PAGE_SIZE - 16) / std::mem::size_of::<PeekTok<'static>>();

        while ctx.peek_cache.len() < PEEK_CACHE_SIZE {
            let Some(token) = ctx.inner.next() else {
                break;
            };

            self.trapped_by_token(ctx, (token.unwrap(), ctx.inner.slice()));
        }

        // Reverse the peek cache to make it a stack
        ctx.peek_cache.reverse();

        // Pop the first token again
        ctx.peeked = ctx.peek_cache.pop();
    }

    #[inline]
    fn trapped_by_token(&mut self, ctx: &mut StreamContext<'a>, (kind, text): PeekTok<'a>) {
        if kind != Token::CommandName(CommandName::Generic) {
            ctx.peek_cache.push((kind, text));
            return;
        }

        let cmd_name = &text[1..];

        // todo: trap begin/end env
        let name = classify(cmd_name);
        if name != CommandName::Generic {
            ctx.peek_cache.push((Token::CommandName(name), text));
            return;
        }

        let Some(m) = self.macros.get(cmd_name) else {
            ctx.peek_cache
                .push((Token::CommandName(CommandName::Generic), text));
            return;
        };

        match m {
            Macro::Declare(_) => todo!(),
            Macro::Cmd(_) => todo!(),
            Macro::Env(_) => todo!(),
        }
    }

    /// Create a new scope for macro definitions
    pub fn create_scope(&mut self) -> Checkpoint {
        let _ = self.env_stack;

        // make self.macros a mutable
        (self.macros.to_mut().snapshot(),)
    }

    /// Restore the scope (delete all macros defined in the child scope)
    pub fn restore(&mut self, (snapshot,): Checkpoint) {
        let _ = self.env_stack;

        self.macros.to_mut().rollback_to(snapshot);
    }

    /// Peek the next token and its text
    pub fn add_macro(&mut self, name: &str, value: &Macro) {
        // self.symbol_table.insert(name.to_owned(), value.to_owned());
        format!("{:?} => {:?}", name, value);
        todo!()
    }
}
