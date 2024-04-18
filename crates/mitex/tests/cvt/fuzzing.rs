use super::prelude::*;

#[test]
fn test_fuzzing() {
    assert_snapshot!(convert_math(r#"\left\0"#).unwrap_err(), @r###"error: unknown command: \0"###);
    assert_snapshot!(convert_math(r#"\end{}"#).unwrap_err(), @r###"error: error unexpected: """###);
}
