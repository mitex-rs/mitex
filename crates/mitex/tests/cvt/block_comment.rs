use super::prelude::*;

#[test]
fn base() {
    assert_debug_snapshot!(convert_text(r#"\iffalse Test\fi"#), @r###"
    Ok(
        "",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\iffalse Test\else \LaTeX\fi"#), @r###"
    Ok(
        " LaTeX ",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\iffalse Test\ifhbox Commented HBox\fi\fi"#), @r###"
    Ok(
        "",
    )
    "###);
}
