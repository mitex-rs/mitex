use super::prelude::*;

#[test]
fn base() {
    assert_debug_snapshot!(parse(r#"\left.\right."#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),word'("."))
    ||clause-lr(cmd-name("\\right"),word'("."))
    "###);
    assert_debug_snapshot!(parse(r#"\left.a\right."#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),word'("."))
    ||text(word'("a"))
    ||clause-lr(cmd-name("\\right"),word'("."))
    "###);
    assert_debug_snapshot!(parse(r#"\left.    \right] ,"#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),word'("."))
    ||space'("    ")
    ||clause-lr(cmd-name("\\right"),rbracket'("]"))
    |space'(" ")
    |text(comma'(","))
    "###);
    assert_debug_snapshot!(parse(r#"\left  . a \right    \|"#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),space'("  "),word'("."))
    ||space'(" ")
    ||text(word'("a"),space'(" "))
    ||clause-lr(cmd-name("\\right"),space'("    "),sym'("\\|"))
    "###);
    assert_debug_snapshot!(parse(r#"\left\langle a\right\|"#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),sym'("\\langle"))
    ||space'(" ")
    ||text(word'("a"))
    ||clause-lr(cmd-name("\\right"),sym'("\\|"))
    "###);
    // Note: this is an invalid expression
    // Error handling
    assert_debug_snapshot!(parse(r#"\left{.}a\right{.}"#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),lbrace'("{"))
    ||text(word'("."))
    |error'(rbrace'("}"))
    |text(word'("a"))
    |cmd(cmd-name("\\right"))
    |curly
    ||lbrace'("{")
    ||text(word'("."))
    ||rbrace'("}")
    "###);
    // Note: this is an invalid expression
    // Error handling
    assert_debug_snapshot!(parse(r#"\begin{equation}\left.\right\end{equation}"#), @r###"
    root
    |env
    ||begin(sym'("equation"))
    ||lr
    |||clause-lr(cmd-name("\\left"),word'("."))
    |||clause-lr(cmd-name("\\right"))
    ||end(sym'("equation"))
    "###);
    // Note: this is an invalid expression
    // Error handling
    assert_debug_snapshot!(parse(r#"\begin{equation}\left\right\end{equation}"#), @r###"
    root
    |env
    ||begin(sym'("equation"))
    ||lr
    |||clause-lr(cmd-name("\\left"))
    |||clause-lr(cmd-name("\\right"))
    ||end(sym'("equation"))
    "###);
}
