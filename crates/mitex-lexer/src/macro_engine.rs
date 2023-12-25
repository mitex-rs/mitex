//!
//! Good reference
//! - <https://latexref.xyz/>
//! - <https://en.wikibooks.org/wiki/Category:Book:TeX>
//! - <https://www.tug.org/utilities/plain/cseq.html>
//!
//! Commands Supported
//!
//! - \newcommand
//! - \newcommand*
//! - \renewcommand
//! - \renewcommand*
//! - \DeclareRobustCommand
//! - \DeclareRobustCommand*
//! - \providecommand
//! - \providecommand*
//!
//! Commands in plan
//!
//! - \DeclareTextCommand
//! - \DeclareTextCommandDefault
//! - \ProvideTextCommand
//! - \ProvideTextCommandDefault
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
    snapshot_map::{self, SnapshotMap},
    BraceKind, BumpTokenStream, CommandName, MacroifyStream, StreamContext, Tok, Token,
};
use mitex_spec::CommandSpec;

pub type Checkpoint = (snapshot_map::Snapshot,);

type MacroMap<'a> = SnapshotMap<&'a str, Macro<'a>>;

#[derive(Debug)]
pub struct CmdMacro<'a> {
    pub name: String,
    pub num_args: u8,
    pub opt: Option<Vec<Tok<'a>>>,
    pub def: Vec<Tok<'a>>,
}

#[derive(Debug)]
pub struct EnvMacro<'a> {
    pub name: String,
    pub num_args: u8,
    pub opt: Option<Vec<Tok<'a>>>,
    pub begin_def: Vec<Tok<'a>>,
    pub end_def: Vec<Tok<'a>>,
}

#[derive(Debug, Clone)]
pub enum DeclareCmdOrEnv {
    /// Command macro for NewCommand/RenewCommand{*}
    /// Synopsis, one of:
    ///
    /// \{re}newcommand{*}{\cmd}{defn}
    /// \{re}newcommand{*}{\cmd}[nargs]{defn}
    /// \{re}newcommand{*}{\cmd}[nargs][optargdefault]{defn}
    NewCommand { renew: bool, star: bool },
    /// Command macro for ProvideCommand{*}
    /// Synopsis, one of:
    ///
    /// \providecommand{*}{\cmd}{defn}
    /// \providecommand{*}{\cmd}[nargs]{defn}
    /// \providecommand{*}{\cmd}[nargs][optargdefault]{defn}
    ProvideCommand { star: bool },
    /// Command macro for DeclareRobustCommand{*}
    /// Synopsis, one of:
    ///
    /// \DeclareRobustCommand{*}{\cmd}{defn}
    /// \DeclareRobustCommand{*}{\cmd}[nargs]{defn}
    /// \DeclareRobustCommand{*}{\cmd}[nargs][optargdefault]{defn}
    DeclareRobustCommand { star: bool },
    /// Command macro for NewEnvironment/RenewEnvironment{*}
    /// Synopsis, one of:
    ///
    /// \{re}newenvironment{*}{env}{begdef}{enddef}
    /// \{re}newenvironment{*}{env}[nargs]{begdef}{enddef}
    /// \{re}newenvironment{*}{env}[nargs][optargdefault]{begdef}{enddef}
    NewEnvironment { renew: bool, star: bool },
}

#[derive(Debug, Clone)]
pub enum DeclareMacro {
    CmdOrEnv(DeclareCmdOrEnv),
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
        (
            "newcommand",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewCommand {
                renew: false,
                star: false,
            }),
        ),
        (
            "newcommand*",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewCommand {
                renew: false,
                star: true,
            }),
        ),
        (
            "renewcommand",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewCommand {
                renew: true,
                star: false,
            }),
        ),
        (
            "renewcommand*",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewCommand {
                renew: true,
                star: true,
            }),
        ),
        (
            "providecommand",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::ProvideCommand { star: false }),
        ),
        (
            "providecommand*",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::ProvideCommand { star: true }),
        ),
        (
            "DeclareRobustCommand",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::DeclareRobustCommand { star: false }),
        ),
        (
            "DeclareRobustCommand*",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::DeclareRobustCommand { star: true }),
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
        (
            "newenvironment",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewEnvironment {
                renew: false,
                star: false,
            }),
        ),
        (
            "newenvironment*",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewEnvironment {
                renew: false,
                star: true,
            }),
        ),
        (
            "renewenvironment",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewEnvironment {
                renew: true,
                star: false,
            }),
        ),
        (
            "renewenvironment*",
            DeclareMacro::CmdOrEnv(DeclareCmdOrEnv::NewEnvironment {
                renew: true,
                star: true,
            }),
        ),
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
    macros: Cow<'a, MacroMap<'a>>,
    /// Environment stack
    env_stack: Vec<EnvMacro<'a>>,
    /// Macro stack
    pub reading_macro: Vec<MacroNode<'a>>,
    /// Toekns used by macro stack
    pub scanned_tokens: Vec<Tok<'a>>,
}

impl<'a> BumpTokenStream<'a> for MacroEngine<'a> {
    fn bump(&mut self, ctx: &mut StreamContext<'a>) {
        self.do_bump(ctx);
    }
}

impl<'a> MacroifyStream<'a> for MacroEngine<'a> {
    fn get_macro(&self, name: &str) -> Option<Macro<'a>> {
        self.macros.get(name).cloned()
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
        const PEEK_CACHE_SIZE: usize = (PAGE_SIZE - 16) / std::mem::size_of::<Tok<'static>>();
        /// Reserve one item for the peeked token
        const PEEK_CACHE_SIZE_M1: usize = PEEK_CACHE_SIZE - 1;

        ctx.next_token();
        while ctx.peek_outer.buf.len() < PEEK_CACHE_SIZE_M1 {
            let Some(token) = ctx.peek_full() else {
                break;
            };

            self.trapped_by_token(ctx, token);
        }

        if let Some(e) = ctx.peek_full() {
            ctx.push_outer(e);
        }

        // Reverse the peek cache to make it a stack
        ctx.peek_outer.buf.reverse();

        // Pop the first token again
        ctx.peek_outer.peeked = ctx.peek_outer.buf.pop();
    }

    #[inline]
    fn trapped_by_token(
        &mut self,
        ctx: &mut StreamContext<'a>,
        (kind, text): Tok<'a>,
    ) -> Option<()> {
        if kind != Token::CommandName(CommandName::Generic) {
            ctx.push_outer((kind, text));
            ctx.next_token();
            return None;
        }

        let cmd_name = &text[1..];

        let Some(m) = self.macros.get(cmd_name) else {
            ctx.push_outer((Token::CommandName(CommandName::Generic), text));
            ctx.next_token();
            return None;
        };

        use DeclareMacro::*;
        match m {
            Macro::Declare(CmdOrEnv(c)) => {
                // {\cmd}[nargs][optargdefault]{defn}

                ctx.next_not_trivia()
                    .filter(|nx| *nx == Token::Left(BraceKind::Curly))?;
                ctx.next_not_trivia();

                let name = ctx
                    .peek_cmd_name_opt(BraceKind::Curly)?
                    .1
                    .strip_prefix('\\')
                    .unwrap();

                #[derive(Clone, Copy, PartialEq)]
                enum MatchState {
                    NArgs,
                    OptArgDefault,
                    DefN,
                }

                let mut state = MatchState::NArgs;

                let mut num_args: u8 = 0;
                let mut opt = None;
                let def;
                'match_loop: loop {
                    let nx = ctx.peek()?;

                    match (state, nx) {
                        (MatchState::NArgs, Token::Left(BraceKind::Bracket)) => {
                            ctx.next_not_trivia();
                            num_args = ctx.peek_u8_opt(BraceKind::Bracket).filter(|e| *e <= 9)?;
                            state = MatchState::OptArgDefault;
                        }
                        (MatchState::OptArgDefault, Token::Left(BraceKind::Bracket)) => {
                            ctx.next_token();
                            opt = Some(ctx.read_until_balanced(BraceKind::Bracket));
                            state = MatchState::DefN;
                        }
                        (_, Token::Left(BraceKind::Curly)) => {
                            ctx.next_token();
                            def = ctx.read_until_balanced(BraceKind::Curly);
                            break 'match_loop;
                        }
                        _ => {
                            def = vec![ctx.peek_full().unwrap()];
                            ctx.next_token();
                            break 'match_loop;
                        }
                    }
                }

                enum UpdateAction {
                    New,
                    Renew,
                    Provide,
                }

                let mut is_env = false;
                let mut end_def = None;
                let action = match c {
                    DeclareCmdOrEnv::NewCommand { renew, star: _ } => {
                        if *renew {
                            UpdateAction::Renew
                        } else {
                            UpdateAction::New
                        }
                    }
                    DeclareCmdOrEnv::DeclareRobustCommand { star: _ } => UpdateAction::New,
                    DeclareCmdOrEnv::ProvideCommand { star: _ } => UpdateAction::Provide,
                    DeclareCmdOrEnv::NewEnvironment { renew, star: _ } => {
                        is_env = true;

                        if matches!(ctx.peek()?, Token::Left(BraceKind::Curly)) {
                            end_def = Some(ctx.read_until_balanced(BraceKind::Curly));
                        }

                        if *renew {
                            UpdateAction::Renew
                        } else {
                            UpdateAction::New
                        }
                    }
                };

                let def = Self::process_macro_def(def);

                let m = if is_env {
                    let end_def = end_def.map(|e| Self::process_macro_def(e))?;
                    Macro::Env(Arc::new(EnvMacro {
                        name: name.to_owned(),
                        num_args,
                        opt,
                        begin_def: def,
                        end_def,
                    }))
                } else {
                    Macro::Cmd(Arc::new(CmdMacro {
                        name: name.to_owned(),
                        num_args,
                        opt,
                        def,
                    }))
                };

                // todo: improve performance
                match action {
                    UpdateAction::New => {
                        if self.get_macro(name).is_some() {
                            ctx.push_outer((Token::Error, name));
                        }

                        self.add_macro(name, m);
                    }
                    UpdateAction::Renew => {
                        if self.get_macro(name).is_none() {
                            ctx.push_outer((Token::Error, name));
                        }

                        self.add_macro(name, m);
                    }
                    UpdateAction::Provide => {
                        if self.get_macro(name).is_none() {
                            self.add_macro(name, m);
                        }
                    }
                }

                None
            }
            Macro::Declare(
                DeclareTextCommand
                | ProvideTextCommand
                | DeclareTextCommandDefault
                | ProvideTextCommandDefault,
            ) => {
                ctx.push_outer((kind, text));
                ctx.next_token();
                None
            }
            Macro::Declare(AtEndOfClass | AtEndOfPackage | AtBeginDocument | AtEndDocument) => {
                ctx.push_outer((kind, text));
                ctx.next_token();
                None
            }
            Macro::Cmd(cmd) => {
                let args = Self::read_macro_args(ctx, cmd.num_args, cmd.opt.clone())?;
                // expand macro args
                let mut stream = vec![];

                let mut i = 0;
                let mut bc = 0;
                while i < cmd.def.len() {
                    let e = &cmd.def[i];
                    match e.0 {
                        Token::MacroArg(num) => {
                            if let Some(arg) = args.get(num as usize - 1) {
                                stream.extend(arg.iter().cloned());
                            }
                        }
                        Token::CommandName(CommandName::Generic) => {
                            let name = e.1.strip_prefix('\\').unwrap();
                            match name {
                                "mitexrecurse" => loop {
                                    i += 1;
                                    if i >= cmd.def.len() {
                                        break;
                                    }
                                    let e = &cmd.def[i];
                                    if e.0 == Token::Left(BraceKind::Curly) {
                                        if bc > 0 {
                                            stream.push(*e);
                                        }
                                        bc += 1;
                                    } else if e.0 == Token::Right(BraceKind::Curly) {
                                        bc -= 1;
                                        if bc == 0 {
                                            break;
                                        } else {
                                            stream.push(*e);
                                        }
                                    } else if bc != 0 {
                                        stream.push(*e);
                                    } else if !e.0.is_trivia() {
                                        stream.push(*e);
                                        break;
                                    }
                                },
                                _ => stream.push(*e),
                            }
                        }
                        _ => stream.push(*e),
                    }
                    i += 1;
                }

                ctx.extend_inner(stream.into_iter().rev());
                if ctx.peek_inner.peeked.is_none() {
                    ctx.next_token();
                }

                None
            }
            Macro::Env(_) => {
                ctx.push_outer((kind, text));
                ctx.next_token();
                None
            }
        }
    }

    fn read_macro_args(
        ctx: &mut StreamContext<'a>,
        num_args: u8,
        opt: Option<Vec<Tok<'a>>>,
    ) -> Option<Vec<Vec<Tok<'a>>>> {
        let mut args = Vec::with_capacity(num_args as usize);

        ctx.next_token();
        let mut num_of_read: u8 = 0;
        loop {
            match ctx.peek_not_trivia() {
                Some(Token::Left(BraceKind::Curly)) => {
                    ctx.next_token();
                    args.push(ctx.read_until_balanced(BraceKind::Curly));
                }
                Some(Token::Word) => {
                    let t = ctx.peek_full().unwrap().1;
                    let mut split_cnt = 0;
                    for c in t.chars() {
                        args.push(vec![(Token::Word, &t[split_cnt..split_cnt + c.len_utf8()])]);
                        split_cnt += c.len_utf8();
                        num_of_read += 1;
                        if num_of_read == num_args {
                            break;
                        }
                    }
                    if split_cnt < t.len() {
                        ctx.peek_inner.peeked.as_mut().unwrap().1 = &t[split_cnt..];
                    } else {
                        ctx.next_token();
                    }
                    if num_of_read == num_args {
                        break;
                    }
                }
                Some(_) => {
                    args.push(vec![ctx.peek_full().unwrap()]);
                    ctx.next_token();
                }
                None => {
                    break;
                }
            }

            num_of_read += 1;
            if num_of_read == num_args {
                break;
            }
        }

        if num_of_read != num_args {
            let mut ok = false;
            if num_args - num_of_read == 1 {
                if let Some(opt) = opt {
                    args.push(opt);
                    ok = true;
                }
            }

            if !ok {
                ctx.push_outer((Token::Error, "invalid number of arguments"));
                return None;
            }
        }

        Some(args)
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
    pub fn add_macro(&mut self, name: &'a str, value: Macro<'a>) {
        self.macros.to_mut().insert(name, value);
    }

    fn process_macro_def(mut def: Vec<(Token, &str)>) -> Vec<(Token, &str)> {
        // process hash, it will grab the next token
        let mut empty_texts = false;
        for i in 0..def.len() {
            if def[i].0 == Token::Hash {
                let next = def.get_mut(i + 1).unwrap();
                if next.0 == Token::Word {
                    let Some(first_char) = next.1.chars().next() else {
                        continue;
                    };

                    if first_char.is_ascii_digit() {
                        let Some(num) = first_char.to_digit(10) else {
                            continue;
                        };
                        if num == 0 {
                            continue;
                        }
                        if num > 9 {
                            panic!("macro argument number must be in range 1..9");
                        }
                        next.1 = &next.1[1..];
                        if next.1.is_empty() {
                            empty_texts = true;
                        }
                        def[i].0 = Token::MacroArg(num as u8);
                    }
                }
            }
        }

        if !empty_texts {
            return def;
        }

        def.retain(|e| e.0 != Token::Word || !e.1.is_empty());
        def
    }
}
