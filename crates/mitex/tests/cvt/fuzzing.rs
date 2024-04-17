use super::prelude::*;

#[test]
fn test_fuzzing() {
    assert_debug_snapshot!(convert_math(r#"\left\0"#), @r###"
    Err(
        "error: unknown command: \\0",
    )
    "###);
    assert_debug_snapshot!(convert_math(r#"\end{}"#), @r###"
    Err(
        "error: error unexpected: \"\"",
    )
    "###);
}
