use super::prelude::*;

#[test]
fn test_fuzzing() {
    assert_debug_snapshot!(parse(r#"\left\0"#), @r###"
    root
    |lr
    ||clause-lr(cmd-name("\\left"),sym'("\\0"))
    "###);
    assert_debug_snapshot!(parse(r#"\end{}"#), @r###"
    root
    |error'(sym'(""))
    "###);
}
