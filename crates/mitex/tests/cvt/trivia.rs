use super::prelude::*;

#[test]
fn curly_group() {
    assert_snapshot!(convert_text(r#"a \mathbf{strong} text"#).unwrap(), @"a mitexmathbf[strong]; text");
}

#[test]
fn arguments() {
    assert_snapshot!(convert_text(r#"\frac { 1 } { 2 }"#).unwrap(), @"frac[ 1 ];[ 2 ];");
}

#[test]
fn greedy_trivia() {
    assert_snapshot!(convert_text(r#"a {\displaystyle text } b"#).unwrap(), @"a mitexdisplay( text ) b");
    assert_snapshot!(convert_text(r#"\displaystyle text "#).unwrap(), @"mitexdisplay( text )");
    assert_snapshot!(convert_text(r#"\displaystyle {text} "#).unwrap(), @"mitexdisplay( text, )");
    assert_snapshot!(convert_text(r#"\displaystyle {\mathrm {text}} "#).unwrap(), @"mitexdisplay( upright[text];, )");
}
