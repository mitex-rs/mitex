use logos::{Logos, Source};
mod macro_engine;
pub mod snapshot_map;

use macro_engine::Macro;
use mitex_spec::CommandSpec;

pub use macro_engine::MacroEngine;

/// A peeked token
type PeekTok<'a> = (Token, &'a str);

#[derive(Debug, Clone)]
pub struct PeekCache<'a> {
    /// Prepare for token peeking
    /// The last peeked token
    pub peeked: Option<PeekTok<'a>>,
    /// A set of peeked tokens takes up to one page of memory
    /// It also takes CPU locality into consideration
    pub peek_cache: Vec<PeekTok<'a>>,
}

impl Default for PeekCache<'_> {
    fn default() -> Self {
        Self {
            peeked: None,
            peek_cache: Vec::with_capacity(8),
        }
    }
}

/// A stream context for [`Lexer`]
#[derive(Debug, Clone)]
pub struct StreamContext<'a> {
    /// Input source
    /// The inner lexer
    pub inner: logos::Lexer<'a, Token>,

    /// Outer peek
    pub peek_outer: PeekCache<'a>,
    /// Internal peek
    peek_inner: PeekCache<'a>,
}

impl<'a> StreamContext<'a> {
    #[inline]
    fn next_token_inner(inner: &mut logos::Lexer<'a, Token>) -> Option<Token> {
        inner.next().map(|e| {
            let tok = e.unwrap();

            if let Token::CommandName(CommandName::Generic) = tok {
                Token::CommandName(classify(&inner.slice()[1..]))
            } else {
                tok
            }
        })
    }

    #[inline]
    fn next_full(&mut self) -> Option<PeekTok<'a>> {
        self.next_token();
        self.peek_inner.peeked
    }

    #[inline]
    fn peek_full(&mut self) -> Option<PeekTok<'a>> {
        self.peek_inner.peeked
    }

    // Inner bumping is not cached
    #[inline]
    pub fn next_token(&mut self) {
        if let Some(peeked) = self.peek_inner.peek_cache.pop() {
            self.peek_inner.peeked = Some(peeked);
            return;
        }

        self.peek_inner.peeked =
            Self::next_token_inner(&mut self.inner).map(|e| (e, self.inner.slice()));
    }

    fn peek_not_trivia(&mut self) -> Option<Token> {
        self.peek()
            .into_iter()
            .chain(std::iter::from_fn(|| self.next_full().map(|(e, _)| e)))
            .find(|e| !e.is_trivia())
    }

    fn next_not_trivia(&mut self) -> Option<Token> {
        std::iter::from_fn(|| self.next_full().map(|(e, _)| e)).find(|e| !e.is_trivia())
    }

    fn peek(&mut self) -> Option<Token> {
        self.peek_inner.peeked.map(|(kind, _)| kind)
    }

    fn eat_if(&mut self, tk: Token) {
        if self.peek_inner.peeked.map_or(false, |e| e.0 == tk) {
            self.next_token();
        }
    }

    fn read_u8_option(&mut self, bk: BraceKind) -> Option<u8> {
        let until_tok = Token::Right(bk);
        let (nx, t) = self.peek_full()?;
        if nx != Token::Word {
            self.next_not_trivia();
            return None;
        }

        let w: u64 = t.parse().ok()?;

        self.next_not_trivia()?;
        self.eat_if(until_tok);
        (w <= 9).then_some(w as u8)
    }

    fn read_command_name_option(&mut self, bk: BraceKind) -> Option<PeekTok<'a>> {
        let until_tok = Token::Right(bk);
        let res = self.peek_full()?;
        if !matches!(res.0, Token::CommandName(..)) {
            self.next_not_trivia();
            return None;
        }

        self.next_not_trivia()?;
        self.eat_if(until_tok);
        Some(res)
    }

    fn read_until_balanced(&mut self, bk: BraceKind) -> Vec<PeekTok<'a>> {
        let until_tok = Token::Right(bk);

        let mut c = 0;
        self.peek_full()
            .into_iter()
            .chain(std::iter::from_fn(|| self.next_full()))
            .take_while(|(e, _)| {
                let e = *e;
                if c == 0 && e == until_tok {
                    false
                } else {
                    if e == Token::Left(BraceKind::Curly) {
                        c += 1;
                    } else if e == Token::Right(BraceKind::Curly) {
                        c -= 1;
                    }
                    true
                }
            })
            .collect()
    }

    fn push_outer(&mut self, peeked: PeekTok<'a>) {
        self.peek_outer.peek_cache.push(peeked);
    }

    fn extend_inner(&mut self, peeked: impl Iterator<Item = PeekTok<'a>>) {
        if let Some(peeked) = self.peek_inner.peeked {
            self.peek_inner.peek_cache.push(peeked);
            self.peek_inner.peeked = None;
        }
        self.peek_inner.peek_cache.extend(peeked);
        self.next_token();
    }
}

/// A trait for bumping the token stream
/// Its bumping is less frequently called than token peeking
pub trait BumpTokenStream<'a> {
    /// Bump the token stream with at least one token if possible
    ///
    /// By default, it fills the peek cache with a page of tokens at the same
    /// time
    fn bump(&mut self, ctx: &mut StreamContext<'a>) {
        default_bump(&mut ctx.inner, &mut ctx.peek_outer)
    }

    fn get_macro(&self, _name: &str) -> Option<Macro<'a>> {
        None
    }
}

/// The default implementation of [`BumpTokenStream`]
///
/// See [`default_bump`] for implementation
impl BumpTokenStream<'_> for () {}

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
        let inner = Token::lexer_with_extras(input, spec);
        let mut n = Self {
            ctx: StreamContext {
                inner,
                peek_outer: PeekCache::default(),
                peek_inner: PeekCache::default(),
            },
            bumper,
        };
        n.next();

        n
    }

    /// Private method to advance the lexer by one token
    #[inline]
    fn next(&mut self) {
        if let Some(peeked) = self.ctx.peek_outer.peek_cache.pop() {
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

/// fills the peek cache with a page of tokens at the same time
fn default_bump<'a>(l: &mut logos::Lexer<'a, Token>, cache: &mut PeekCache<'a>) {
    /// The size of a page, in some architectures it is 16384B but that doesn't
    /// matter
    const PAGE_SIZE: usize = 4096;
    /// The item size of the peek cache
    const PEEK_CACHE_SIZE: usize = (PAGE_SIZE - 16) / std::mem::size_of::<PeekTok<'static>>();

    for _ in 0..PEEK_CACHE_SIZE {
        if let Some(tok) = StreamContext::next_token_inner(l).map(|nx| (nx, l.slice())) {
            cache.peek_cache.push(tok);
        } else {
            break;
        }
    }

    // Reverse the peek cache to make it a stack
    cache.peek_cache.reverse();

    // Pop the first token again
    cache.peeked = cache.peek_cache.pop();
}

/// Classify the command name so parser can use it repeatedly
fn classify(name: &str) -> CommandName {
    match name {
        "begin" => CommandName::BeginEnvironment,
        "end" => CommandName::EndEnvironment,
        "iffalse" => CommandName::BeginBlockComment,
        "fi" => CommandName::EndBlockComment,
        "left" => CommandName::Left,
        "right" => CommandName::Right,
        _ => CommandName::Generic,
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
#[logos(extras = CommandSpec)]
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

/// Lex a valid command name
// todo: handle commands with underscores, whcih would require command names
// todo: from specification
fn lex_command_name(lexer: &mut logos::Lexer<Token>) -> CommandName {
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

    /// The utf8 length of ascii chars
    const LEN_ASCII: usize = 1;

    // Case3 (Rest): lex a general ascii command name
    // We treat the command name as ascii to improve performance slightly
    let ascii_str = command_start.as_bytes()[LEN_ASCII..].iter();

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
                let spec = &lexer.extras;
                let mut s = lexer.span();
                // for char `\`
                s.start += 1;
                // for char  `*`
                s.end += 1;
                let name = lexer.source().slice(s);
                if name.and_then(|s| spec.get(s)).is_some() {
                    lexer.bump(LEN_ASCII);
                }

                break;
            }
            c if c.is_ascii_alphabetic() => lexer.bump(LEN_ASCII),
            // todo: math mode don't want :
            // b'@' | b':' => lexer.bump(LEN_ASCII),
            b'@' => lexer.bump(LEN_ASCII),
            _ => break,
        };
    }

    CommandName::Generic
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
    /// clause of BlockComment: \iffalse
    BeginBlockComment,
    /// clause of BlockComment: \fi
    EndBlockComment,
    /// clause of LRItem: \left
    Left,
    /// clause of LRItem: \right
    Right,
}
