use super::prelude::*;

/// Argument will reset flag of being in a formula
#[test]
fn arg_scope() {
    assert_debug_snapshot!(parse(r#"$\text{${1}$}$"#), @r###"
    root
    |formula
    ||dollar'("$")
    ||cmd
    |||cmd-name("\\text")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||formula
    ||||||dollar'("$")
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("1"))
    |||||||rbrace'("}")
    ||||||dollar'("$")
    |||||rbrace'("}")
    ||dollar'("$")
    "###);
    // Note: This is a valid AST, but semantically incorrect (indicated by overleaf)
    assert_debug_snapshot!(parse(r#"$\frac{${1}$}{${2}$}$"#), @r###"
    root
    |formula
    ||dollar'("$")
    ||cmd
    |||cmd-name("\\frac")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||formula
    ||||||dollar'("$")
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("1"))
    |||||||rbrace'("}")
    ||||||dollar'("$")
    |||||rbrace'("}")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||formula
    ||||||dollar'("$")
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("2"))
    |||||||rbrace'("}")
    ||||||dollar'("$")
    |||||rbrace'("}")
    ||dollar'("$")
    "###);
}
