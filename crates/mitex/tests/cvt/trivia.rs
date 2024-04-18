use super::prelude::*;

#[test]
fn curly_group() {
    assert_snapshot!(convert_math(r#"a \mathbf{strong} text"#).unwrap(), @"a  mitexmathbf(s t r o n g ) t e x t ");
}

#[test]
fn arguments() {
    assert_snapshot!(convert_math(r#"\frac { 1 } { 2 }"#).unwrap(), @"frac( 1  , 2  )");
}

#[test]
fn greedy_trivia() {
    assert_snapshot!(convert_math(r#"a {\displaystyle text } b"#).unwrap(), @"a  mitexdisplay( t e x t  ) b ");
    assert_snapshot!(convert_math(r#"\displaystyle text "#).unwrap(), @"mitexdisplay( t e x t  )");
    assert_snapshot!(convert_math(r#"\displaystyle {text} "#).unwrap(), @"mitexdisplay( t e x t , )");
    assert_snapshot!(convert_math(r#"\displaystyle {\mathrm {text}} "#).unwrap(), @"mitexdisplay( upright(t e x t ), )");
}
