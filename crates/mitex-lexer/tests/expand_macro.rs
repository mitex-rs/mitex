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
fn begin_end() {
    assert_snapshot!(assert_plain_tokens(r#"\begin{eq}\end{eq}"#), @r###"
    CommandName(BeginEnvironment)("eq")
    CommandName(EndEnvironment)("eq")
    "###);
    assert_snapshot!(assert_plain_tokens(r#"\begin{align*}\end{align*}"#), @r###"
    CommandName(BeginEnvironment)("align*")
    CommandName(EndEnvironment)("align*")
    "###);
}

#[test]
fn no_macros() {
    assert_snapshot!(assert_plain_tokens("hello world"), @r###"
    Word("hello")
    Whitespace(" ")
    Word("world")
    "###);
    assert_snapshot!(assert_plain_tokens("{a#1a}"), @r###"
    Left(Curly)("{")
    Word("a")
    Hash("#")
    Word("1a")
    Right(Curly)("}")
    "###);
}

// collect all tokens until eat() returns None
fn get_macro(input: &str, macro_name: &str) -> String {
    let mut lexer = Lexer::new_with_bumper(
        input,
        DEFAULT_SPEC.clone(),
        MacroEngine::new(DEFAULT_SPEC.clone()),
    );
    while lexer.eat().is_some() {
        continue;
    }

    match lexer.get_macro(macro_name) {
        Some(e) => {
            assert_eq!(tokens(input), "");
            format!("{:#?}", e)
        }
        None => {
            format!("FailedRest:{}", tokens(input))
        }
    }
}

#[test]
fn ignoring_unimplemented() {
    assert_snapshot!(r#"\AtEndOfClass{code}"#, @r###"\AtEndOfClass{code}"###);
}

#[test]
fn bug_playground() {}

#[test]
fn declare_macro() {
    assert_snapshot!(get_macro(r#"\newcommand{\mytheta}{\theta}"#, "mytheta"), @r###"
    Cmd(
        CmdMacro {
            name: "mytheta",
            num_args: 0,
            opt: None,
            def: [
                (
                    CommandName(
                        Generic,
                    ),
                    "\\theta",
                ),
            ],
        },
    )
    "###);
    assert_snapshot!(get_macro(r#"\newcommand{\mytheta}[4]{\theta}"#, "mytheta"), @r###"
    Cmd(
        CmdMacro {
            name: "mytheta",
            num_args: 4,
            opt: None,
            def: [
                (
                    CommandName(
                        Generic,
                    ),
                    "\\theta",
                ),
            ],
        },
    )
    "###);
    assert_snapshot!(get_macro(r#"\newcommand{\mytheta}[10]{\theta}"#, "mytheta"), @r###"
    FailedRest:Left(Curly)("{")
    CommandName(Generic)("\\theta")
    Right(Curly)("}")
    "###);
    assert_snapshot!(get_macro(r#"\newcommand{\mytheta}[4][  \orz]{\theta}"#, "mytheta"), @r###"
    Cmd(
        CmdMacro {
            name: "mytheta",
            num_args: 4,
            opt: Some(
                [
                    (
                        Whitespace,
                        "  ",
                    ),
                    (
                        CommandName(
                            Generic,
                        ),
                        "\\orz",
                    ),
                ],
            ),
            def: [
                (
                    CommandName(
                        Generic,
                    ),
                    "\\theta",
                ),
            ],
        },
    )
    "###);
}

#[test]
fn subst_macro() {
    assert_snapshot!(tokens(r#"\newcommand{\f}[2]{#1f(#2)}\f\hat xy"#), @r###"
    CommandName(Generic)("\\hat")
    Word("f")
    Left(Paren)("(")
    Word("x")
    Right(Paren)(")")
    Word("y")
    "###);
    assert_snapshot!(tokens(r#"\newenvironment{f}[2]{begin}{end}\begin{f}test\end{f}"#), @r###"
    Word("begin")
    Word("st")
    Word("end")
    "###);
}

#[test]
fn newcommand_recursive() {
    assert_snapshot!(tokens(r#"\newcommand{\DeclareMathDelimit}[2]{\newcommand{#1}[1]{\left#2\mitexrecurse{#1}\right#2}}\DeclareMathDelimit{\abs}{\vert}\abs{abc}"#), @r###"
    CommandName(Left)("\\left")
    CommandName(Generic)("\\vert")
    Word("abc")
    CommandName(Right)("\\right")
    CommandName(Generic)("\\vert")
    "###);
}
