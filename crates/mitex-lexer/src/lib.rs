use logos::{Logos, Source};
mod macro_engine;
pub mod snapshot_map;

use macro_engine::Macro;
use mitex_spec::CommandSpec;

pub use macro_engine::MacroEngine;

/// MiTeX's token representation
/// A token is a pair of a token kind and its text
type Tok<'a> = (Token, &'a str);

/// Lex Cache for bundling (bumping) lexing operations for CPU locality
#[derive(Debug, Clone)]
pub struct LexCache<'a> {
    /// The last peeked token
    pub peeked: Option<Tok<'a>>,
    /// A reversed sequence of peeked tokens
    pub buf: Vec<Tok<'a>>,
}

impl Default for LexCache<'_> {
    fn default() -> Self {
        Self {
            peeked: None,
            buf: Vec::with_capacity(8),
        }
    }
}

impl<'a> LexCache<'a> {
    /// Extend the peek cache with a sequence of tokens
    ///
    /// Note: the tokens in the cache are reversed
    fn extend(&mut self, peeked: impl Iterator<Item = Tok<'a>>) {
        // Push the peeked token back to the peek cache
        let peeking = if let Some(peeked) = self.peeked {
            self.buf.push(peeked);
            true
        } else {
            false
        };

        self.buf.extend(peeked);

        // Pop the first token again
        if peeking {
            self.peeked = self.buf.pop();
        }
    }

    /// Fill the peek cache with a page of tokens at the same time
    fn bump(&mut self, peeked: impl Iterator<Item = Tok<'a>>) {
        assert!(
            self.buf.is_empty(),
            "all of tokens in the peek cache should be consumed before bumping",
        );

        /// The size of a page, in some architectures it is 16384B but that
        /// doesn't matter, we only need a sensible value
        const PAGE_SIZE: usize = 4096;
        /// The item size of the peek cache
        const PEEK_CACHE_SIZE: usize = (PAGE_SIZE - 16) / std::mem::size_of::<Tok<'static>>();

        // Fill the peek cache with a page of tokens
        self.buf.extend(peeked.take(PEEK_CACHE_SIZE));
        // Reverse the peek cache to make it a stack
        self.buf.reverse();
        // Pop the first token again
        self.peeked = self.buf.pop();
    }
}

/// A stream context for [`Lexer`]
#[derive(Debug, Clone)]
pub struct StreamContext<'a> {
    /// Input source
    /// The inner lexer
    pub inner: logos::Lexer<'a, Token>,

    /// Outer peek
    pub peek_outer: LexCache<'a>,
    /// Inner peek
    peek_inner: LexCache<'a>,
}

impl<'a> StreamContext<'a> {
    #[inline]
    fn lex_one(l: &mut logos::Lexer<'a, Token>) -> Option<Tok<'a>> {
        let tok = l.next()?.unwrap();

        let source_text = match tok {
            Token::CommandName(CommandName::BeginEnvironment | CommandName::EndEnvironment) => {
                l.source().slice(l.extras.1.clone()).unwrap()
            }
            _ => l.slice(),
        };

        Some((tok, source_text))
    }

    // Inner bumping is not cached
    #[inline]
    pub fn next_token(&mut self) {
        let peeked = self
            .peek_inner
            .buf
            .pop()
            .or_else(|| Self::lex_one(&mut self.inner));
        self.peek_inner.peeked = peeked;
    }

    #[inline]
    fn next_full(&mut self) -> Option<Tok<'a>> {
        self.next_token();
        self.peek_inner.peeked
    }

    #[inline]
    fn peek_full(&mut self) -> Option<Tok<'a>> {
        self.peek_inner.peeked
    }

    fn peek(&mut self) -> Option<Token> {
        self.peek_inner.peeked.map(|(kind, _)| kind)
    }

    #[inline]
    fn next_stream(&mut self) -> impl Iterator<Item = Tok<'a>> + '_ {
        std::iter::from_fn(|| self.next_full())
    }

    #[inline]
    fn peek_stream(&mut self) -> impl Iterator<Item = Tok<'a>> + '_ {
        self.peek_full().into_iter().chain(self.next_stream())
    }

    fn next_not_trivia(&mut self) -> Option<Token> {
        self.next_stream().map(|e| e.0).find(|e| !e.is_trivia())
    }

    fn peek_not_trivia(&mut self) -> Option<Token> {
        self.peek_stream().map(|e| e.0).find(|e| !e.is_trivia())
    }

    fn eat_if(&mut self, tk: Token) {
        if self.peek_inner.peeked.map_or(false, |e| e.0 == tk) {
            self.next_token();
        }
    }

    fn push_outer(&mut self, peeked: Tok<'a>) {
        self.peek_outer.buf.push(peeked);
    }

    fn extend_inner(&mut self, peeked: impl Iterator<Item = Tok<'a>>) {
        self.peek_inner.extend(peeked);
    }

    fn peek_u8_opt(&mut self, bk: BraceKind) -> Option<u8> {
        let res = self
            .peek_full()
            .filter(|res| matches!(res.0, Token::Word))
            .and_then(|(_, text)| text.parse().ok());
        self.next_not_trivia()?;

        self.eat_if(Token::Right(bk));

        res
    }

    fn peek_word_opt(&mut self, bk: BraceKind) -> Option<Tok<'a>> {
        let res = self.peek_full().filter(|res| matches!(res.0, Token::Word));
        self.next_not_trivia()?;

        self.eat_if(Token::Right(bk));

        res
    }

    fn peek_cmd_name_opt(&mut self, bk: BraceKind) -> Option<Tok<'a>> {
        let res = self
            .peek_full()
            .filter(|res| matches!(res.0, Token::CommandName(..)));

        self.next_not_trivia()?;
        self.eat_if(Token::Right(bk));

        res
    }

    fn read_until_balanced(&mut self, bk: BraceKind) -> Vec<Tok<'a>> {
        let until_tok = Token::Right(bk);

        let mut curly_level = 0;
        let match_curly = &mut |e: Token| {
            if curly_level == 0 && e == until_tok {
                return false;
            }

            match e {
                Token::Left(BraceKind::Curly) => curly_level += 1,
                Token::Right(BraceKind::Curly) => curly_level -= 1,
                _ => {}
            }

            true
        };

        let res = self
            .peek_stream()
            .take_while(|(e, _)| match_curly(*e))
            .collect();

        self.eat_if(until_tok);
        res
    }
}

/// A trait for bumping the token stream
/// Its bumping is less frequently called than token peeking
pub trait BumpTokenStream<'a>: MacroifyStream<'a> {
    /// Bump the token stream with at least one token if possible
    ///
    /// By default, it fills the peek cache with a page of tokens at the same
    /// time
    fn bump(&mut self, ctx: &mut StreamContext<'a>) {
        ctx.peek_outer.bump(std::iter::from_fn(|| {
            StreamContext::lex_one(&mut ctx.inner)
        }));
    }
}

/// Trait for querying macro state of a stream
pub trait MacroifyStream<'a> {
    /// Get a macro by name (if meeted in the stream)
    fn get_macro(&self, _name: &str) -> Option<Macro<'a>> {
        None
    }
}

/// The default implementation of [`BumpTokenStream`]
///
/// See [`LexCache<'a>`] for implementation
impl BumpTokenStream<'_> for () {}

/// The default implementation of [`MacroifyStream`]
impl MacroifyStream<'_> for () {}

/// Small memory-efficient lexer for TeX
///
/// It gets improved performance on x86_64 but not wasm through
#[derive(Debug, Clone)]
pub struct Lexer<'a, S: BumpTokenStream<'a> = ()> {
    /// A stream context shared with the bumper
    ctx: StreamContext<'a>,
    /// Implementations to bump the token stream into [`Self::ctx`]
    bumper: S,
}

impl<'a, S: BumpTokenStream<'a>> Lexer<'a, S> {
    /// Create a new lexer on a main input source
    ///
    /// Note that since we have a bumper, the returning string is not always
    /// sliced from the input
    pub fn new(input: &'a str, spec: CommandSpec) -> Self
    where
        S: Default,
    {
        Self::new_with_bumper(input, spec, S::default())
    }

    /// Create a new lexer on a main input source with a bumper
    ///
    /// Note that since we have a bumper, the returning string is not always
    /// sliced from the input
    pub fn new_with_bumper(input: &'a str, spec: CommandSpec, bumper: S) -> Self {
        let inner = Token::lexer_with_extras(input, (spec, 0..0));
        let mut n = Self {
            ctx: StreamContext {
                inner,
                peek_outer: LexCache::default(),
                peek_inner: LexCache::default(),
            },
            bumper,
        };
        n.next();

        n
    }

    /// Private method to advance the lexer by one token
    #[inline]
    fn next(&mut self) {
        if let Some(peeked) = self.ctx.peek_outer.buf.pop() {
            self.ctx.peek_outer.peeked = Some(peeked);
            return;
        }

        // it is not likely to be inlined
        self.bumper.bump(&mut self.ctx);
    }

    /// Peek the next token
    pub fn peek(&self) -> Option<Token> {
        self.ctx.peek_outer.peeked.map(|(kind, _)| kind)
    }

    /// Peek the next token's text
    pub fn peek_text(&self) -> Option<&'a str> {
        self.ctx.peek_outer.peeked.map(|(_, text)| text)
    }

    /// Peek the next token's first char
    pub fn peek_char(&self) -> Option<char> {
        self.peek_text().map(str::chars).and_then(|mut e| e.next())
    }

    /// Update the text part of the peeked token
    pub fn consume_utf8_bytes(&mut self, cnt: usize) {
        let Some(peek_mut) = &mut self.ctx.peek_outer.peeked else {
            return;
        };
        if peek_mut.1.len() <= cnt {
            self.next();
        } else {
            peek_mut.1 = &peek_mut.1[cnt..];
        }
    }

    /// Update the peeked token and return the old one
    pub fn eat(&mut self) -> Option<(Token, &'a str)> {
        let peeked = self.ctx.peek_outer.peeked.take()?;
        self.next();
        Some(peeked)
    }

    pub fn get_macro(&mut self, name: &str) -> Option<Macro<'a>> {
        self.bumper.get_macro(name)
    }
}

/// Brace kinds in TeX, used by defining [`Token`]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum BraceKind {
    /// Curly braces: `{` or `}`
    Curly,
    /// brackets: `[` or `]`
    Bracket,
    /// Parenthesis: `(` or `)`
    Paren,
}

/// Mark the brace kind of a token as curly
#[inline(always)]
fn bc(_: &mut logos::Lexer<Token>) -> BraceKind {
    BraceKind::Curly
}

/// Mark the brace kind of a token as bracket
#[inline(always)]
fn bb(_: &mut logos::Lexer<Token>) -> BraceKind {
    BraceKind::Bracket
}

/// Mark the brace kind of a token as parenthesis
#[inline(always)]
fn bp(_: &mut logos::Lexer<Token>) -> BraceKind {
    BraceKind::Paren
}

/// The token types defined in logos
///
/// For naming of marks, see <https://en.wikipedia.org/wiki/List_of_typographical_symbols_and_punctuation_marks>
///
/// It also specifies how logos would lex the token
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
#[logos(extras = (CommandSpec, logos::Span))]
pub enum Token {
    #[regex(r"[\r\n]+", priority = 2)]
    LineBreak,

    #[regex(r"[^\S\r\n]+", priority = 1)]
    Whitespace,

    #[regex(r"%[^\r\n]*")]
    LineComment,

    #[token("{", bc)]
    #[token("[", bb)]
    #[token("(", bp)]
    Left(BraceKind),

    #[token("}", bc)]
    #[token("]", bb)]
    #[token(")", bp)]
    Right(BraceKind),

    #[token(",")]
    Comma,

    #[token("~")]
    Tilde,

    #[token("/")]
    Slash,

    #[token("&")]
    Ampersand,

    #[token("^")]
    Caret,

    #[token("'")]
    Apostrophe,

    #[token("\"")]
    Ditto,

    #[token(";")]
    Semicolon,

    #[token("#")]
    Hash,

    #[token("_", priority = 2)]
    Underscore,

    #[regex(r#"[^\s\\%\{\},\$\[\]\(\)\~/_'";&^#]+"#, priority = 1)]
    Word,

    #[regex(r"\$\$?")]
    Dollar,

    /// Though newline is also a valid command, whose name is `\`, we lex it
    /// independently so to help later AST consumers. This also means that user
    /// cannot redefine `\` as a command.
    #[regex(r"\\\\", priority = 4)]
    NewLine,

    #[regex(r"\\", lex_command_name, priority = 3)]
    CommandName(CommandName),

    /// Macro error
    Error,

    MacroArg(u8),
}

impl Token {
    pub fn is_trivia(&self) -> bool {
        use Token::*;
        matches!(self, LineBreak | Whitespace | LineComment)
    }
}

/// The utf8 length of ascii chars
const LEN_ASCII: usize = 1;

/// Lex a valid command name
// todo: handle commands with underscores, whcih would require command names
// todo: from specification
fn lex_command_name(lexer: &mut logos::Lexer<Token>) -> CommandName {
    use IfCommandName::*;
    let command_start = &lexer.source()[lexer.span().end..];

    // Get the first char in utf8 case
    let c = match command_start.chars().next() {
        Some(c) => c,
        None => return CommandName::Generic,
    };

    // Case1: `\ ` is not a command name hence the command is empty
    // Note: a space is not a command name
    if c.is_whitespace() {
        return CommandName::Generic;
    }

    // Case2: `\.*` is a command name, e.g. `\;` is a space command in TeX
    // Note: the first char is always legal, since a backslash with any single char
    // is a valid escape sequence
    lexer.bump(c.len_utf8());
    // Lex the command name if it is not an escape sequence
    if !c.is_ascii_alphabetic() && c != '@' {
        return CommandName::Generic;
    }

    // Case3 (Rest): lex a general ascii command name
    // We treat the command name as ascii to improve performance slightly
    let ascii_str = &command_start.as_bytes()[LEN_ASCII..];
    let bump_size = advance_ascii_name(lexer, ascii_str, true);
    lexer.bump(bump_size);

    let name = &command_start[..LEN_ASCII + bump_size];
    match name {
        "if" => CommandName::If(If),
        "iftypst" => CommandName::If(IfTypst),
        "iffalse" => CommandName::If(IfFalse),
        "iftrue" => CommandName::If(IfTrue),
        "ifcase" => CommandName::If(IfCase),
        "ifnum" => CommandName::If(IfNum),
        "ifcat" => CommandName::If(IfCat),
        "ifx" => CommandName::If(IfX),
        "ifvoid" => CommandName::If(IfVoid),
        "ifhbox" => CommandName::If(IfHBox),
        "ifvbox" => CommandName::If(IfVBox),
        "ifhmode" => CommandName::If(IfHMode),
        "ifmmode" => CommandName::If(IfMMode),
        "ifvmode" => CommandName::If(IfVMode),
        "ifinner" => CommandName::If(IfInner),
        "ifdim" => CommandName::If(IfDim),
        "ifeof" => CommandName::If(IfEof),
        "@ifstar" => CommandName::If(IfStar),
        "else" => CommandName::Else,
        "fi" => CommandName::EndIf,
        "left" => CommandName::Left,
        "right" => CommandName::Right,
        "begin" => lex_begin_end(lexer, true),
        "end" => lex_begin_end(lexer, false),
        _ => CommandName::Generic,
    }
}

fn advance_ascii_name(
    lexer: &mut logos::Lexer<Token>,
    ascii_str: &[u8],
    lex_slash_command: bool,
) -> usize {
    let mut bump_size = 0;
    for c in ascii_str {
        match c {
            // Find the command name in the spec
            // If a starred command is not found, recover to a normal command
            // This is the same behavior as TeX
            //
            // We can build a regex set to improve performance
            // but overall this is not a bottleneck so we don't do it now
            // And RegexSet heavily increases the binary size
            b'*' => {
                let verified = if lex_slash_command {
                    let spec = &lexer.extras.0;
                    // for char `\`, etc.
                    let s = lexer.span().start + 1;
                    // for char  `*`
                    let s = s..s + bump_size + 2;
                    let t = lexer.source().slice(s);
                    t.and_then(|s| spec.get(s)).is_some()
                } else {
                    true
                };

                if verified {
                    bump_size += LEN_ASCII;
                }

                break;
            }
            c if c.is_ascii_alphabetic() => bump_size += LEN_ASCII,
            // todo: math mode don't want :
            // b'@' | b':' => bump_size += LEN_ASCII,
            b'@' => bump_size += LEN_ASCII,
            _ => break,
        };
    }

    bump_size
}

fn lex_begin_end(lexer: &mut logos::Lexer<Token>, is_begin: bool) -> CommandName {
    struct LexTask<'a, 'b> {
        lexer: &'a mut logos::Lexer<'b, Token>,
        chars: std::str::Chars<'b>,
        collected: usize,
    }

    impl<'a, 'b> LexTask<'a, 'b> {
        fn new(lexer: &'a mut logos::Lexer<'b, Token>) -> Self {
            Self {
                chars: lexer.source()[lexer.span().end..].chars(),
                lexer,
                collected: 0,
            }
        }

        fn next_non_trivia(&mut self) -> Option<char> {
            loop {
                let c = match self.chars.next() {
                    Some(c) => c,
                    None => break None,
                };

                if c.is_whitespace() {
                    self.collected += c.len_utf8();
                    continue;
                }

                if c == '%' {
                    self.collected += c.len_utf8();
                    for c in self.chars.by_ref() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        self.collected += c.len_utf8();
                    }
                    continue;
                }

                self.collected += c.len_utf8();
                return Some(c);
            }
        }

        #[inline(always)]
        fn work(&mut self) -> Option<()> {
            let c = self.next_non_trivia()?;

            if c != '{' {
                return None;
            }

            let ns = self.lexer.span().end + self.collected;
            let ascii_str = self.lexer.source()[ns..].as_bytes();

            let bump_size = advance_ascii_name(self.lexer, ascii_str, false);
            self.lexer.extras.1 = ns..ns + bump_size;
            self.collected += bump_size;
            self.chars = self.lexer.source()[ns + bump_size..].chars();

            let c = self.next_non_trivia()?;
            if c != '}' {
                return None;
            }

            self.lexer.bump(self.collected);
            Some(())
        }
    }

    let mut task = LexTask::new(lexer);
    match (task.work(), is_begin) {
        (Some(..), true) => CommandName::BeginEnvironment,
        (Some(..), false) => CommandName::EndEnvironment,
        (None, true) => CommandName::ErrorBeginEnvironment,
        (None, false) => CommandName::ErrorEndEnvironment,
    }
}

/// The command name used by parser
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum IfCommandName {
    /// \if
    If,
    /// \iftypst
    IfTypst,
    /// \iffalse
    IfFalse,
    /// \iftrue
    IfTrue,
    /// \ifcase
    IfCase,
    /// \ifnum
    IfNum,
    /// \ifcat
    IfCat,
    /// \ifx
    IfX,
    /// \ifvoid
    IfVoid,
    /// \ifhbox
    IfHBox,
    /// \ifvbox
    IfVBox,
    /// \ifhmode
    IfHMode,
    /// \ifmmode
    IfMMode,
    /// \ifvmode
    IfVMode,
    /// \ifinner
    IfInner,
    /// \ifdim
    IfDim,
    /// \ifeof
    IfEof,
    /// \@ifstar
    IfStar,
}

/// The command name used by parser
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CommandName {
    /// Rest of the command names
    Generic,
    /// clause of Environment: \begin
    BeginEnvironment,
    /// clause of Environment: \end
    EndEnvironment,
    /// clause of Environment: \begin, but error
    ErrorBeginEnvironment,
    /// clause of Environment: \end, but error
    ErrorEndEnvironment,
    /// clause of IfStatements: \if...
    If(IfCommandName),
    /// clause of IfStatements: \else
    Else,
    /// clause of IfStatements: \fi
    EndIf,
    /// clause of LRItem: \left
    Left,
    /// clause of LRItem: \right
    Right,
}
