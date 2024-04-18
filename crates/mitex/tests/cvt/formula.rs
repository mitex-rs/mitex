use super::prelude::*;

#[test]
fn cmd_formula() {
    assert_snapshot!(convert_text(r#"\[\]"#).unwrap(), @"$  $");
    // Note: this is a valid AST, but semantically incorrect
    assert_snapshot!(convert_text(r#"\[\[\]\]"#).unwrap(), @"$  $");
    // Note: this is a valid AST, but semantically incorrect
    assert_snapshot!(convert_text(r#"\[\(\)\]"#).unwrap(), @"$  $");
    // Note: this is a valid AST, but semantically incorrect
    // It looks strange, but we regard it as a valid AST for simplicity
    assert_snapshot!(convert_text(r#"\[\)\(\]"#).unwrap_err(), @"error: formula is not valid");
}

#[test]
fn formula_scope() {
    assert_snapshot!(convert_text(r#"$[)$ test"#).unwrap(), @r###"#math.equation(block: false, $\[\)$); test"###);
}

#[test]
fn curly_scope() {
    // Note: this is a broken AST
    assert_snapshot!(convert_text(r#"${$}"#).unwrap_err(), @"error: formula is not valid");
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_snapshot!(convert_text(r#"{$}$"#).unwrap(), @"#math.equation(block: false, $$);#math.equation(block: false, $$);");
}

#[test]
fn env_scope() {
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_snapshot!(convert_text(r#"\begin{array}$\end{array}$"#).unwrap(), @"$ mitexarray(arg0: ,) $#math.equation(block: false, $$);");
    // Note: this is a valid but incompleted AST, converter should handle it
    // correctly
    assert_snapshot!(convert_text(r#"$\begin{array}$\end{array}"#).unwrap_err(), @"error: formula is not valid");
}
