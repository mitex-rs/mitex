use super::prelude::*;

#[test]
fn base() {
    assert_debug_snapshot!(parse(r#"\iffalse Test\fi"#), @r###"
    root
    |block-comment(space'(" "),word'("Test"))
    "###);
    assert_debug_snapshot!(parse(r#"\iffalse Test\else \LaTeX\fi"#), @r###"
    root
    |block-comment(space'(" "),word'("Test"))
    |space'(" ")
    |cmd(cmd-name("\\LaTeX"))
    "###);
    assert_debug_snapshot!(parse(r#"\iffalse Test\ifhbox Commented HBox\fi\fi"#), @r###"
    root
    |block-comment(space'(" "),word'("Test"),cmd-name("\\ifhbox"),space'(" "),word'("Commented"),space'(" "),word'("HBox"),cmd-name("\\fi"))
    "###);
}
