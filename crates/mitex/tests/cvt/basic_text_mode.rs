use super::prelude::*;

#[test]
fn test_convert_text_mode() {
    assert_snapshot!(convert_text(r#"abc"#).unwrap(), @"abc");
    assert_snapshot!(convert_text(r#"\section{Title}"#).unwrap(), @"#heading(level: 1)[Title];");
    assert_snapshot!(convert_text(r#"a \textbf{strong} text"#).unwrap(), @"a #strong[strong]; text");
}
