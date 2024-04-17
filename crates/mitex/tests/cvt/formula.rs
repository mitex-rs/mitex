use super::prelude::*;

#[test]
fn cmd_formula() {
    assert_debug_snapshot!(convert_text(r#"\[\]"#), @r###"
    Ok(
        "$  $",
    )
    "###);
    // Note: this is a valid AST, but semantically incorrect
    assert_debug_snapshot!(convert_text(r#"\[\[\]\]"#), @r###"
    Ok(
        "$  $",
    )
    "###);
    // Note: this is a valid AST, but semantically incorrect
    assert_debug_snapshot!(convert_text(r#"\[\(\)\]"#), @r###"
    Ok(
        "$  $",
    )
    "###);
    // Note: this is a valid AST, but semantically incorrect
    // It looks strange, but we regard it as a valid AST for simplicity
    assert_debug_snapshot!(convert_text(r#"\[\)\(\]"#), @r###"
    Err(
        "error: formula is not valid",
    )
    "###);
}

#[test]
fn formula_scope() {
    assert_debug_snapshot!(convert_text(r#"$[)$ test"#), @r###"
    Ok(
        "#math.equation(block: false, $\\[\\)$); test",
    )
    "###);
}

#[test]
fn curly_scope() {
    // Note: this is a broken AST
    assert_debug_snapshot!(convert_text(r#"${$}"#), @r###"
    Err(
        "error: formula is not valid",
    )
    "###);
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_debug_snapshot!(convert_text(r#"{$}$"#), @r###"
    Ok(
        "#math.equation(block: false, $$);#math.equation(block: false, $$);",
    )
    "###);
}

#[test]
fn env_scope() {
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_debug_snapshot!(convert_text(r#"\begin{array}$\end{array}$"#), @r###"
    Ok(
        "$ mitexarray(arg0: ,) $#math.equation(block: false, $$);",
    )
    "###);
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_debug_snapshot!(convert_text(r#"$\begin{array}$\end{array}"#), @r###"
    Err(
        "error: formula is not valid",
    )
    "###);
}
