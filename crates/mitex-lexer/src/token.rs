use logos::{Logos, Source};
use mitex_spec::CommandSpec;

/// Brace kinds in TeX, used by defining [`Token`]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum BraceKind {
    /// Curly braces: `{` or `}`
    Curly,
    /// brackets (Square braces): `[` or `]`
    Bracket,
    /// Parenthesis: `(` or `)`
    Paren,
}

/// The token types defined in logos
///
/// For naming of marks, see <https://en.wikipedia.org/wiki/List_of_typographical_symbols_and_punctuation_marks>
///
/// It also specifies how logos would lex the token
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
#[logos(extras = (CommandSpec, logos::Span))]
pub enum Token {
    /// A line break
    /// Typically a `\r\n` or `\n`
    #[regex(r"[\r\n]+", priority = 2)]
    LineBreak,

    /// A whitespace sequence that doesn't contain line breaks
    /// Typically a space or a tab
    #[regex(r"[^\S\r\n]+", priority = 1)]
    Whitespace,

    /// A comment that follows a line break
    /// E.g.
    ///
    /// ```tex
    /// % This is a comment
    /// ```
    #[regex(r"%[^\r\n]*")]
    LineComment,

    /// Left braces
    /// E.g. `{`, `[`, `(`, etc.
    /// See [`BraceKind`] for braces.
    #[token("{", bc)]
    #[token("[", bb)]
    #[token("(", bp)]
    Left(BraceKind),

    /// Right braces
    /// E.g. `}`, `]`, `)`, etc.
    /// See [`BraceKind`] for braces.
    #[token("}", bc)]
    #[token("]", bb)]
    #[token(")", bp)]
    Right(BraceKind),

    /// An ascii comma
    #[token(",")]
    Comma,

    /// An ascii tilde
    #[token("~")]
    Tilde,

    /// An ascii slash
    #[token("/")]
    Slash,

    /// An ascii ampersand
    #[token("&")]
    Ampersand,

    /// An ascii caret
    #[token("^")]
    Caret,

    /// An ascii apostrophe
    #[token("'")]
    Apostrophe,

    /// An ascii ditto
    #[token("\"")]
    Ditto,

    /// An ascii semicolon
    #[token(";")]
    Semicolon,

    /// An ascii hash
    #[token("#")]
    Hash,

    /// An ascii asterisk
    #[token("*")]
    Asterisk,

    /// An ascii atsign
    #[token("@")]
    AtSign,

    /// An ascii underscore
    #[token("_", priority = 2)]
    Underscore,

    /// A character sequence that doesn't contain any above tokens
    #[regex(r#"[^\s\\%\{\},\$\[\]\(\)\~/_\*@'";&^#]+"#, priority = 1)]
    Word,

    /// Special dollar signs
    #[regex(r"\$\$?")]
    Dollar,

    /// Though newline is also a valid command, whose name is `\`, we lex it
    /// independently so to help later AST consumers. This also means that user
    /// cannot redefine `\` as a command.
    #[regex(r"\\\\", priority = 4)]
    NewLine,

    /// A command start with a backslash
    /// Note: backslash (`\`) is a command without name
    /// Note: An escape sequence is a command with any single unicode char
    #[regex(r"\\", lex_command_name, priority = 3)]
    CommandName(CommandName),

    /// Macro error
    Error,

    /// A macro argument
    MacroArg(u8),
}

impl Token {
    /// Determine whether the token is trivia
    pub fn is_trivia(&self) -> bool {
        use Token::*;
        matches!(self, LineBreak | Whitespace | LineComment)
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
    /// clause of Math: \( or \[
    BeginMath,
    /// clause of Math: \) or \]
    EndMath,
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
    match c {
        '(' | '[' => return CommandName::BeginMath,
        ')' | ']' => return CommandName::EndMath,
        '@' => {}
        _ if !c.is_ascii_alphabetic() => return CommandName::Generic,
        _ => {}
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
