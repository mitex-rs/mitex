use super::prelude::*;

#[test]
fn base() {
    // println!("{:#?}", parse(r#"{}_{1}^1"#));
    assert_snapshot!(convert_math(r#"_1^2"#).unwrap(), @"zws_(1 )zws^(2 )");
    assert_snapshot!(convert_math(r#"{}_{1}^2"#).unwrap(), @"zws _(1 )^(2 )");
    assert_snapshot!(convert_math(r#"\alpha_1"#).unwrap(), @"alpha _(1 )");
    assert_snapshot!(convert_math(r#"\alpha_[1]"#).unwrap(), @r###"alpha _(\[)1 \]"###);
    assert_snapshot!(convert_math(r#"\alpha_(1)"#).unwrap(), @r###"alpha _(\()1 \)"###);
    assert_snapshot!(convert_math(r#"_1"#).unwrap(), @"zws_(1 )");
    // Note: this is an invalid expression
    assert_snapshot!(convert_math(r#"\over_1"#).unwrap(), @"frac(,zws_(1 ))");
    assert_snapshot!(convert_math(r#"{}_1"#).unwrap(), @"zws _(1 )");
    assert_snapshot!(convert_math(r#"{}_1_1"#).unwrap(), @"zws _(1 )_(1 )");
    assert_snapshot!(convert_math(r#"\frac{1}{2}_{3}"#).unwrap(), @"frac(1 ,2 )_(3 )");
    assert_snapshot!(convert_math(r#"\overbrace{a + b + c}^{\text{This is an overbrace}}"#).unwrap(), @"mitexoverbrace(a  +  b  +  c )^(#textmath[This is an overbrace];)");
    assert_snapshot!(convert_math(r#"\underbrace{x \times y}_{\text{This is an underbrace}}"#).unwrap(), @"mitexunderbrace(x  times  y )_(#textmath[This is an underbrace];)");
    assert_snapshot!(convert_math(r#"x_1''^2"#).unwrap(), @"x _(1 )''^(2 )");
    assert_snapshot!(convert_math(r#"x''_1"#).unwrap(), @"x ''_(1 )");
    assert_snapshot!(convert_math(r#"''"#).unwrap(), @"''");
    assert_snapshot!(convert_math(r#"\frac''"#).unwrap(), @"frac(',')");
}

#[test]
fn test_attachment_may_weird() {
    assert_snapshot!(convert_math(r#"\frac ab_c"#).unwrap(), @"frac(a ,b )_(c )");
    assert_snapshot!(convert_math(r#"\frac a_c b"#).unwrap(), @"frac(a )_(c ) b ");
    assert_snapshot!(convert_math(r#"\frac {a_c} b"#).unwrap(), @"frac(a _(c ),b )");
}
