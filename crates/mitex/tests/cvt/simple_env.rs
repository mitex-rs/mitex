use super::prelude::*;

#[test]
fn quote() {
    assert_snapshot!(convert_text(r#"\begin{quote}\end{quote}"#).unwrap(), @"#quote(block: true)[];");
    assert_snapshot!(convert_text(r#"\begin{quote}yes\end{quote}"#).unwrap(), @"#quote(block: true)[yes];");
}

#[test]
fn test_abstract() {
    assert_snapshot!(convert_text(r#"\begin{abstract}\end{abstract}"#).unwrap(), @"#quote(block: true)[];");
    assert_snapshot!(convert_text(r#"\begin{abstract}yes\end{abstract}"#).unwrap(), @"#quote(block: true)[yes];");
}
