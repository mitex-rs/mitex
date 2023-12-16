extern crate logos;

use crate::syntax::SyntaxKind;
use logos::Logos;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<(Token, &'a str)>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut tokens = tokenize(input);
        tokens.reverse();
        Self { tokens }
    }

    pub fn peek(&self) -> Option<Token> {
        self.tokens.last().map(|(kind, _)| *kind)
    }

    pub fn eat(&mut self) -> Option<(SyntaxKind, &'a str)> {
        let (kind, text) = self.tokens.pop()?;
        let kind = match kind {
            Token::LineBreak => SyntaxKind::LINE_BREAK,
            Token::Whitespace => SyntaxKind::WHITESPACE,
            Token::LineComment => SyntaxKind::COMMENT,
            Token::LCurly => SyntaxKind::L_CURLY,
            Token::RCurly => SyntaxKind::R_CURLY,
            Token::LBrack => SyntaxKind::L_BRACK,
            Token::RBrack => SyntaxKind::R_BRACK,
            Token::LParen => SyntaxKind::L_PAREN,
            Token::RParen => SyntaxKind::R_PAREN,
            Token::Comma => SyntaxKind::COMMA,
            Token::Eq => SyntaxKind::EQUALITY_SIGN,
            Token::Pipe => SyntaxKind::WORD,
            Token::Word => SyntaxKind::WORD,
            Token::Dollar => SyntaxKind::DOLLAR,
            Token::CommandName(_) => SyntaxKind::COMMAND_NAME,
        };

        Some((kind, text))
    }
}

fn tokenize(input: &str) -> Vec<(Token, &str)> {
    let mut lexer = Token::lexer(input);
    std::iter::from_fn(move || {
        let kind = lexer.next()?.unwrap();
        let text = lexer.slice();
        Some((kind, text))
    })
    .map(|(kind, text)| {
        if kind == Token::CommandName(CommandName::Generic) {
            let name = classify(&text[1..]);
            (Token::CommandName(name), text)
        } else {
            (kind, text)
        }
    })
    .collect()
}

pub fn classify(name: &str) -> CommandName {
    match name {
        "begin" => CommandName::BeginEnvironment,
        "end" => CommandName::EndEnvironment,
        "[" => CommandName::BeginEquation,
        "]" => CommandName::EndEquation,
        "DeclareMathOperator" | "DeclareMathOperator*" => CommandName::MathOperator,
        "color" | "colorbox" | "textcolor" | "pagecolor" => CommandName::ColorReference,
        "iffalse" => CommandName::BeginBlockComment,
        "fi" => CommandName::EndBlockComment,
        _ => CommandName::Generic,
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum Token {
    #[regex(r"[\r\n]+", priority = 2)]
    LineBreak,

    #[regex(r"[^\S\r\n]+", priority = 1)]
    Whitespace,

    #[regex(r"%[^\r\n]*")]
    LineComment,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("[")]
    LBrack,

    #[token("]")]
    RBrack,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token("=")]
    Eq,

    #[token("|")]
    Pipe,

    #[regex(r"[^\s\\%\{\},\$\[\]\(\)=\|]+")]
    Word,

    #[regex(r"\$\$?")]
    Dollar,

    #[regex(r"\\", lex_command_name)]
    CommandName(CommandName),
}

fn lex_command_name(lexer: &mut logos::Lexer<Token>) -> CommandName {
    let input = &lexer.source()[lexer.span().end..];

    let mut chars = input.chars();
    let Some(c) = chars.next() else {
        return CommandName::Generic;
    };

    if c.is_whitespace() {
        return CommandName::Generic;
    }

    lexer.bump(c.len_utf8());
    if !c.is_alphanumeric() && c != '@' {
        return CommandName::Generic;
    }

    for c in chars {
        match c {
            '*' => {
                lexer.bump(c.len_utf8());
                break;
            }
            c if c.is_alphanumeric() => {
                lexer.bump(c.len_utf8());
            }
            '@' | ':' | '_' => {
                lexer.bump(c.len_utf8());
            }
            _ => {
                break;
            }
        };
    }

    CommandName::Generic
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CommandName {
    Generic,
    BeginEnvironment,
    EndEnvironment,
    BeginEquation,
    EndEquation,
    MathOperator,
    ColorReference,
    BeginBlockComment,
    EndBlockComment,
}
