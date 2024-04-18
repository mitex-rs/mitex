use super::prelude::*;

#[test]
fn base() {
    assert_snapshot!(convert_text(r#"\iffalse Test\fi"#).unwrap(), @"");
    assert_snapshot!(convert_text(r#"\iffalse Test\else \LaTeX\fi"#).unwrap(), @" LaTeX ");
    assert_snapshot!(convert_text(r#"\iffalse Test\ifhbox Commented HBox\fi\fi"#).unwrap(), @"");
}
