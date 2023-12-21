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

use crate::{snapshot_map::SnapshotMap, Lexer, PeekTok, Token};
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
    Cmd(Arc<CmdMacro<'a>>),
    Env(Arc<EnvMacro<'a>>),
}

#[derive(Debug)]
pub enum MacroifyToken<'a> {
    Token(PeekTok<'a>),
    Cmd {
        reading: Arc<CmdMacro<'a>>,
        has_read_arg_protect: u8,
        has_read: u32,
    },
    EnvBegin {
        reading: Arc<EnvMacro<'a>>,
        has_read_arg_protect: u8,
        has_read: u32,
    },
    EnvEnd {
        reading: Arc<EnvMacro<'a>>,
        has_read_arg_protect: u8,
        has_read: u32,
    },
    ArgSlot(Range<usize>),
    ReadingTok(Range<usize>),
}

/// MacroEngine has exact same interface as Lexer, but it expands macros.
///
/// When it meets a macro in token stream, It evaluates a macro into expanded
/// tokens.
pub struct MacroEngine<'a> {
    /// Lexer level structure
    lexer: Lexer<'a>,
    /// Scoped unified symbol table
    symbol_table: MacroMap<'a>,
    /// Global macros in chain
    globals: MacroMap<'a>,
    /// Environment stack
    env_stack: Vec<EnvMacro<'a>>,
    /// Macro stack
    pub reading_macro: Vec<MacroifyToken<'a>>,
}

impl<'a> MacroEngine<'a> {
    /// Create a new macro evaluator
    pub fn new(input: &'a str, spec: CommandSpec) -> Self {
        Self {
            lexer: Lexer::new(input, spec),
            symbol_table: SnapshotMap::default(),
            globals: MacroMap::default(),
            env_stack: Vec::new(),
            reading_macro: Vec::new(),
        }
    }

    /// Peek the next token
    pub fn peek(&self) -> Option<Token> {
        self.lexer.peek()
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
