use super::prelude::*;

#[test]
fn quote() {
    assert_debug_snapshot!(convert_text(r#"\begin{quote}\end{quote}"#), @r###"
    Ok(
        "#quote(block: true)[];",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\begin{quote}yes\end{quote}"#), @r###"
    Ok(
        "#quote(block: true)[yes];",
    )
    "###);
}

#[test]
fn test_abstract() {
    assert_debug_snapshot!(convert_text(r#"\begin{abstract}\end{abstract}"#), @r###"
    Ok(
        "#quote(block: true)[];",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\begin{abstract}yes\end{abstract}"#), @r###"
    Ok(
        "#quote(block: true)[yes];",
    )
    "###);
}
