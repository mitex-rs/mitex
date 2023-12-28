//! Given source strings, MiTeX Lexer provides a sequence of tokens
//!
//! The core of the lexer is [`Lexer<'a, S>`] which receives a string `&'a str`
//! and a [`TokenStream`] trait object `S`, then it provides public methods to
//! peek and bump the token stream.
//!
//! It has two main lexer implementations:
//! - [`Lexer<()>`]: provides plain tokens
//!   - See [`TokenStream`] for implementation
//! - [`Lexer<MacroEngine>`]: provides tokens with macro expansion
//!   - See [`MacroEngine`] for implementation

mod macro_engine;
pub mod snapshot_map;
mod stream;
mod token;

pub use macro_engine::MacroEngine;
pub use token::{BraceKind, CommandName, IfCommandName, Token};

use logos::Logos;
use mitex_spec::CommandSpec;

use macro_engine::Macro;
use stream::{LexCache, StreamContext};

/// MiTeX's token representation
/// A token is a pair of a token kind and its text
type Tok<'a> = (Token, &'a str);

/// A trait for bumping the token stream
/// Its bumping is less frequently called than token peeking
pub trait TokenStream<'a>: MacroifyStream<'a> {
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

/// The default implementation of [`TokenStream`]
///
/// See [`LexCache<'a>`] for implementation
impl TokenStream<'_> for () {}

/// The default implementation of [`MacroifyStream`]
impl MacroifyStream<'_> for () {}

/// Small memory-efficient lexer for TeX
///
/// It gets improved performance on x86_64 but not wasm through
#[derive(Debug, Clone)]
pub struct Lexer<'a, S: TokenStream<'a> = ()> {
    /// A stream context shared with the bumper
    ctx: StreamContext<'a>,
    /// Implementations to bump the token stream into [`Self::ctx`]
    bumper: S,
}

impl<'a, S: TokenStream<'a>> Lexer<'a, S> {
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

    /// Find a **currently** defined macro by name
    pub fn get_macro(&mut self, name: &str) -> Option<Macro<'a>> {
        self.bumper.get_macro(name)
    }
}
