use super::prelude::*;

#[test]
fn self_referencing_macros() {
    let limit = r#"error: error unexpected: "macro expansion limit""#;
    for input in [
        r"\newcommand{\x}{\x}\x",
        r"\newcommand{\x}{\y}\newcommand{\y}{\x}\x",
        r"\newcommand{\x}[1]{\x{#1#1}}\x{a}",
        r"\newenvironment{e}{\begin{e}}{\end{e}}\begin{e}\end{e}",
    ] {
        assert_eq!(convert_math(input).unwrap_err(), limit, "{input}");
    }
}

#[test]
fn document_with_macros() {
    let text = format!(
        r"\newcommand{{\R}}{{\mathbb{{R}}}}\newenvironment{{thm}}{{}}{{}}\begin{{thm}}{}\end{{thm}}",
        r"Let $x \in \R$. ".repeat(300)
    );
    assert!(convert_text(&text).is_ok());
}
