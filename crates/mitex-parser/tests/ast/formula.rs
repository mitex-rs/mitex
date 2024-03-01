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
