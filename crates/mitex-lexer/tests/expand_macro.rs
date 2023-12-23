mod common;

use insta::assert_snapshot;
use mitex_lexer::{BumpTokenStream, Lexer, MacroEngine};

use common::*;

// collect all tokens until eat() returns None
fn tokens_bumper<'a>(input: &'a str, b: impl BumpTokenStream<'a>) -> String {
    let mut lexer = Lexer::new_with_bumper(input, DEFAULT_SPEC.clone(), b);

    std::iter::from_fn(|| lexer.eat().map(|tok| format!("{:?}({:?})", tok.0, tok.1)))
        .collect::<Vec<_>>()
        .join("\n")
}

// collect all tokens until eat() returns None
fn tokens(input: &str) -> String {
    tokens_bumper(input, MacroEngine::new(DEFAULT_SPEC.clone()))
}

// collect all tokens without macro expansion
fn plain_tokens(input: &str) -> String {
    tokens_bumper(input, ())
}

// collect all tokens without macro expansion
fn assert_plain_tokens(input: &str) -> String {
    let left = plain_tokens(input);
    let right = tokens(input);
    assert_eq!(left, right);

    left
}

#[test]
fn no_macros() {
    assert_snapshot!(assert_plain_tokens("hello world"), @r###"
    Word("hello")
    Whitespace(" ")
    Word("world")
    "###);
}
