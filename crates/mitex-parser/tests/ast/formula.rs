use super::prelude::*;

#[test]
fn cmd_formula() {
    assert_debug_snapshot!(parse(r#"\[\]"#), @r###"
    root
    |formula(begin-math'("\\["),end-math("\\]"))
    "###);
    // Note: this is a valid AST, but semantically incorrect
    assert_debug_snapshot!(parse(r#"\[\[\]\]"#), @r###"
    root
    |formula
    ||begin-math'("\\[")
    ||formula(begin-math'("\\["),end-math("\\]"))
    ||end-math("\\]")
    "###);
    // Note: this is a valid AST, but semantically incorrect
    assert_debug_snapshot!(parse(r#"\[\(\)\]"#), @r###"
    root
    |formula
    ||begin-math'("\\[")
    ||formula(begin-math'("\\("),end-math("\\)"))
    ||end-math("\\]")
    "###);
    // Note: this is a valid AST, but semantically incorrect
    // It looks strange, but we regard it as a valid AST for simplicity
    assert_debug_snapshot!(parse(r#"\[\)\(\]"#), @r###"
    root
    |formula(begin-math'("\\["),end-math("\\)"))
    |formula(begin-math'("\\("),end-math("\\]"))
    "###);
}

#[test]
fn formula_scope() {
    assert_debug_snapshot!(parse(r#"$[)$ test"#), @r###"
    root
    |formula(dollar'("$"),lbracket'("["),rparen'(")"),dollar'("$"))
    |space'(" ")
    |text(word'("test"))
    "###);
}

#[test]
fn curly_scope() {
    // Note: this is a broken AST
    assert_debug_snapshot!(parse(r#"${$}"#), @r###"
    root
    |formula
    ||dollar'("$")
    ||curly
    |||lbrace'("{")
    |||formula(dollar'("$"))
    |||rbrace'("}")
    "###);
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_debug_snapshot!(parse(r#"{$}$"#), @r###"
    root
    |curly
    ||lbrace'("{")
    ||formula(dollar'("$"))
    ||rbrace'("}")
    |formula(dollar'("$"))
    "###);
}

#[test]
fn env_scope() {
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_debug_snapshot!(parse(r#"\begin{array}$\end{array}$"#), @r###"
    root
    |env
    ||begin
    |||sym'("array")
    |||args
    ||||formula(dollar'("$"))
    ||end(sym'("array"))
    |formula(dollar'("$"))
    "###);
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_debug_snapshot!(parse(r#"$\begin{array}$\end{array}"#), @r###"
    root
    |formula
    ||dollar'("$")
    ||env
    |||begin(sym'("array"))
    |||formula(dollar'("$"))
    |||end(sym'("array"))
    "###);
}
