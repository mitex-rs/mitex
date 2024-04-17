use super::prelude::*;

#[test]
fn base() {
    assert_debug_snapshot!(convert_math(r#"\left.\right."#), @r###"
    Ok(
        "lr(  )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\left.a\right."#), @r###"
    Ok(
        "lr( a  )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\left.    \right] ,"#), @r###"
    Ok(
        "lr(     \\] ) \\,",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\left  . a \right    \|"#), @r###"
    Ok(
        "lr(     a       || )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\left\langle a\right\|"#), @r###"
    Ok(
        "lr(angle.l  a || )",
    )
    "###);
    // Note: this is an invalid expression
    // Error handling
    assert_debug_snapshot!(convert_math(r#"\left{.}a\right{.}"#), @r###"
    Err(
        "error: error unexpected: \"}\"",
    )
    "###);
    // Note: this is an invalid expression
    // Error handling
    assert_debug_snapshot!(convert_math(r#"\begin{equation}\left.\right\end{equation}"#), @r###"
    Ok(
        "aligned(lr( ))",
    )
    "###);
    // Note: this is an invalid expression
    // Error handling
    assert_debug_snapshot!(convert_math(r#"\begin{equation}\left\right\end{equation}"#), @r###"
    Ok(
        "aligned(lr())",
    )
    "###);
}
