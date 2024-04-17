use super::prelude::*;

#[test]
fn test_convert_text_mode() {
    assert_debug_snapshot!(convert_text(r#"abc"#), @r###"
    Ok(
        "abc",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\section{Title}"#), @r###"
    Ok(
        "#heading(level: 1)[Title];",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"a \textbf{strong} text"#), @r###"
    Ok(
        "a #strong[strong]; text",
    )
    "###);
}
