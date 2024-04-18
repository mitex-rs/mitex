use super::prelude::*;

#[test]
fn base() {
    assert_snapshot!(convert_math(r#"\left.\right."#).unwrap(), @"lr(  )");
    assert_snapshot!(convert_math(r#"\left.a\right."#).unwrap(), @"lr( a  )");
    assert_snapshot!(convert_math(r#"\left.    \right] ,"#).unwrap(), @r###"lr(     \] ) \,"###);
    assert_snapshot!(convert_math(r#"\left  . a \right    \|"#).unwrap(), @"lr(     a       || )");
    assert_snapshot!(convert_math(r#"\left\langle a\right\|"#).unwrap(), @"lr(angle.l  a || )");
    // Note: this is an invalid expression
    // Error handling
    assert_snapshot!(convert_math(r#"\left{.}a\right{.}"#).unwrap_err(), @r###"error: error unexpected: "}""###);
    // Note: this is an invalid expression
    // Error handling
    assert_snapshot!(convert_math(r#"\begin{equation}\left.\right\end{equation}"#).unwrap(), @"aligned(lr( ))");
    // Note: this is an invalid expression
    // Error handling
    assert_snapshot!(convert_math(r#"\begin{equation}\left\right\end{equation}"#).unwrap(), @"aligned(lr())");
}
