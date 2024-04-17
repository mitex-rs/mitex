use super::prelude::*;

#[test]
fn base() {
    // println!("{:#?}", parse(r#"{}_{1}^1"#));
    assert_debug_snapshot!(convert_math(r#"_1^2"#), @r###"
    Ok(
        "zws_(1 )zws^(2 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"{}_{1}^2"#), @r###"
    Ok(
        "zws _(1 )^(2 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\alpha_1"#), @r###"
    Ok(
        "alpha _(1 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\alpha_[1]"#), @r###"
    Ok(
        "alpha _(\\[)1 \\]",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\alpha_(1)"#), @r###"
    Ok(
        "alpha _(\\()1 \\)",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"_1"#), @r###"
    Ok(
        "zws_(1 )",
    )
    "###);
    // Note: this is an invalid expression
    assert_debug_snapshot!(convert_math(r#"\over_1"#), @r###"
    Ok(
        "frac(,zws_(1 ))",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"{}_1"#), @r###"
    Ok(
        "zws _(1 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"{}_1_1"#), @r###"
    Ok(
        "zws _(1 )_(1 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\frac{1}{2}_{3}"#), @r###"
    Ok(
        "frac(1 ,2 )_(3 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\overbrace{a + b + c}^{\text{This is an overbrace}}"#), @r###"
    Ok(
        "mitexoverbrace(a  +  b  +  c )^(#textmath[This is an overbrace];)",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\underbrace{x \times y}_{\text{This is an underbrace}}"#), @r###"
    Ok(
        "mitexunderbrace(x  times  y )_(#textmath[This is an underbrace];)",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"x_1''^2"#), @r###"
    Ok(
        "x _(1 )''^(2 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"x''_1"#), @r###"
    Ok(
        "x ''_(1 )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"''"#), @r###"
    Ok(
        "''",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\frac''"#), @r###"
    Ok(
        "frac(',')",
    )
    "###);
}

#[test]
fn test_attachment_may_weird() {
    assert_debug_snapshot!(convert_math(r#"\frac ab_c"#), @r###"
    Ok(
        "frac(a ,b )_(c )",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\frac a_c b"#), @r###"
    Ok(
        "frac(a )_(c ) b ",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\frac {a_c} b"#), @r###"
    Ok(
        "frac(a _(c ),b )",
    )
    "###);
}
