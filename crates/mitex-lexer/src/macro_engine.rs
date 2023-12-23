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

use std::{ops::Range, sync::Arc};

use crate::{classify, snapshot_map::SnapshotMap, BumpTokenStream, CommandName, PeekTok, Token};
use mitex_spec::CommandSpec;

pub type Snapshot = ();

type MacroMap<'a> = SnapshotMap<String, Macro<'a>>;

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

#[derive(Debug)]
pub enum Macro<'a> {
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
    /// Scoped unified symbol table
    symbol_table: MacroMap<'a>,
    /// Global macros in chain
    globals: MacroMap<'a>,
    /// Environment stack
    env_stack: Vec<EnvMacro<'a>>,
    /// Macro stack
    pub reading_macro: Vec<MacroNode<'a>>,
    /// Toekns used by macro stack
    pub scanned_tokens: Vec<PeekTok<'a>>,
}

impl<'a> BumpTokenStream<'a> for MacroEngine<'a> {
    fn bump(&mut self, ctx: &mut crate::StreamContext<'a>) {
        self.do_bump(ctx);
    }
}

impl<'a> MacroEngine<'a> {
    /// Create a new macro engine
    pub fn new(spec: CommandSpec) -> Self {
        Self {
            spec,
            symbol_table: SnapshotMap::default(),
            globals: MacroMap::default(),
            env_stack: Vec::new(),
            reading_macro: Vec::new(),
            scanned_tokens: Vec::new(),
        }
    }

    /// fills the peek cache with a page of tokens at the same time
    fn do_bump(&mut self, ctx: &mut crate::StreamContext<'a>) {
        /// The size of a page, in some architectures it is 16384B but that
        /// doesn't matter
        const PAGE_SIZE: usize = 4096;
        /// The item size of the peek cache
        const PEEK_CACHE_SIZE: usize = (PAGE_SIZE - 16) / std::mem::size_of::<PeekTok<'static>>();

        for _ in 0..PEEK_CACHE_SIZE {
            let kind = ctx.inner.next().map(|token| {
                let kind = token.unwrap();
                let text = ctx.inner.slice();
                if kind == Token::CommandName(CommandName::Generic) {
                    let name = classify(&text[1..]);
                    (Token::CommandName(name), text)
                } else {
                    (kind, text)
                }
            });
            if let Some(kind) = kind {
                ctx.peek_cache.push(kind);
            } else {
                break;
            }
        }

        // Reverse the peek cache to make it a stack
        ctx.peek_cache.reverse();

        // Pop the first token again
        ctx.peeked = ctx.peek_cache.pop();
    }

    /// Create a new scope for macro definitions
    pub fn create_scope(&mut self) -> Snapshot {
        // let _ = MacroMap::<'a>::with_log;
        let _ = self.env_stack;
    }

    /// Restore the scope (delete all macros defined in the child scope)
    pub fn restore(&mut self, _snapshot: Snapshot) {}

    /// Peek the next token and its text
    pub fn add_macro(&mut self, name: &str, value: &Macro) {
        // self.symbol_table.insert(name.to_owned(), value.to_owned());
        let _ = self.symbol_table;
        let _ = self.globals;
        format!("{:?} => {:?}", name, value);
        todo!()
    }
}
