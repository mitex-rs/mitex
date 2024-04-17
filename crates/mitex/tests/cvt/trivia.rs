use super::prelude::*;

#[test]
fn curly_group() {
    assert_debug_snapshot!(convert_text(r#"a \mathbf{strong} text"#), @r###"
    Ok(
        "a mitexmathbf[strong]; text",
    )
    "###);
}

#[test]
fn arguments() {
    assert_debug_snapshot!(convert_text(r#"\frac { 1 } { 2 }"#), @r###"
    Ok(
        "frac[ 1 ];[ 2 ];",
    )
    "###);
}

#[test]
fn greedy_trivia() {
    assert_debug_snapshot!(convert_text(r#"a {\displaystyle text } b"#), @r###"
    Ok(
        "a mitexdisplay( text ) b",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\displaystyle text "#), @r###"
    Ok(
        "mitexdisplay( text )",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\displaystyle {text} "#), @r###"
    Ok(
        "mitexdisplay( text, )",
    )
    "###);
    assert_debug_snapshot!(convert_text(r#"\displaystyle {\mathrm {text}} "#), @r###"
    Ok(
        "mitexdisplay( upright[text];, )",
    )
    "###);
}
