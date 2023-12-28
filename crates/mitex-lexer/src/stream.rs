use logos::Source;

use crate::{BraceKind, CommandName, Tok, Token};

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
    pub fn bump(&mut self, peeked: impl Iterator<Item = Tok<'a>>) {
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
    pub peek_inner: LexCache<'a>,
}

impl<'a> StreamContext<'a> {
    #[inline]
    pub fn lex_one(l: &mut logos::Lexer<'a, Token>) -> Option<Tok<'a>> {
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
    pub fn next_full(&mut self) -> Option<Tok<'a>> {
        self.next_token();
        self.peek_inner.peeked
    }

    #[inline]
    pub fn peek_full(&mut self) -> Option<Tok<'a>> {
        self.peek_inner.peeked
    }

    pub fn peek(&mut self) -> Option<Token> {
        self.peek_inner.peeked.map(|(kind, _)| kind)
    }

    #[inline]
    pub fn next_stream(&mut self) -> impl Iterator<Item = Tok<'a>> + '_ {
        std::iter::from_fn(|| self.next_full())
    }

    #[inline]
    pub fn peek_stream(&mut self) -> impl Iterator<Item = Tok<'a>> + '_ {
        self.peek_full().into_iter().chain(self.next_stream())
    }

    pub fn next_not_trivia(&mut self) -> Option<Token> {
        self.next_stream().map(|e| e.0).find(|e| !e.is_trivia())
    }

    pub fn peek_not_trivia(&mut self) -> Option<Token> {
        self.peek_stream().map(|e| e.0).find(|e| !e.is_trivia())
    }

    pub fn eat_if(&mut self, tk: Token) {
        if self.peek_inner.peeked.map_or(false, |e| e.0 == tk) {
            self.next_token();
        }
    }

    pub fn push_outer(&mut self, peeked: Tok<'a>) {
        self.peek_outer.buf.push(peeked);
    }

    pub fn extend_inner(&mut self, peeked: impl Iterator<Item = Tok<'a>>) {
        self.peek_inner.extend(peeked);
    }

    pub fn peek_u8_opt(&mut self, bk: BraceKind) -> Option<u8> {
        let res = self
            .peek_full()
            .filter(|res| matches!(res.0, Token::Word))
            .and_then(|(_, text)| text.parse().ok());
        self.next_not_trivia()?;

        self.eat_if(Token::Right(bk));

        res
    }

    pub fn peek_word_opt(&mut self, bk: BraceKind) -> Option<Tok<'a>> {
        let res = self.peek_full().filter(|res| matches!(res.0, Token::Word));
        self.next_not_trivia()?;

        self.eat_if(Token::Right(bk));

        res
    }

    pub fn peek_cmd_name_opt(&mut self, bk: BraceKind) -> Option<Tok<'a>> {
        let res = self
            .peek_full()
            .filter(|res| matches!(res.0, Token::CommandName(..)));

        self.next_not_trivia()?;
        self.eat_if(Token::Right(bk));

        res
    }

    pub fn read_until_balanced(&mut self, bk: BraceKind) -> Vec<Tok<'a>> {
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
