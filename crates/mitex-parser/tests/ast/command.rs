use super::prelude::*;

#[test]
fn left_association() {
    assert_debug_snapshot!(parse(r#"\sum"#), @r###"
    root
    |cmd(cmd-name("\\sum"))
    "###);
    assert_debug_snapshot!(parse(r#"\sum\limits"#), @r###"
    root
    |cmd
    ||args
    |||cmd(cmd-name("\\sum"))
    ||cmd-name("\\limits")
    "###);
    assert_debug_snapshot!(parse(r#"\sum\limits\limits"#), @r###"
    root
    |cmd
    ||args
    |||cmd
    ||||args
    |||||cmd(cmd-name("\\sum"))
    ||||cmd-name("\\limits")
    ||cmd-name("\\limits")
    "###);
    assert_debug_snapshot!(parse(r#"\sum\limits\sum"#), @r###"
    root
    |cmd
    ||args
    |||cmd(cmd-name("\\sum"))
    ||cmd-name("\\limits")
    |cmd(cmd-name("\\sum"))
    "###);
    assert_debug_snapshot!(parse(r#"\sum\limits\sum\limits"#), @r###"
    root
    |cmd
    ||args
    |||cmd(cmd-name("\\sum"))
    ||cmd-name("\\limits")
    |cmd
    ||args
    |||cmd(cmd-name("\\sum"))
    ||cmd-name("\\limits")
    "###);
    assert_debug_snapshot!(parse(r#"\limits"#), @r###"
    root
    |cmd
    ||args()
    ||cmd-name("\\limits")
    "###);
}

#[test]
fn right_greedy() {
    assert_debug_snapshot!(parse(r#"\displaystyle"#), @r###"
    root
    |cmd(cmd-name("\\displaystyle"))
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle a b c"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||space'(" ")
    ||args
    |||text(word'("a"),space'(" "),word'("b"),space'(" "),word'("c"))
    "###);
    assert_debug_snapshot!(parse(r#"a + {\displaystyle a b} c"#), @r###"
    root
    |text(word'("a"),space'(" "),word'("+"),space'(" "))
    |curly
    ||lbrace'("{")
    ||cmd
    |||cmd-name("\\displaystyle")
    |||space'(" ")
    |||args
    ||||text(word'("a"),space'(" "),word'("b"))
    ||rbrace'("}")
    ||space'(" ")
    |text(word'("c"))
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle \sum T"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||space'(" ")
    ||args
    |||cmd(cmd-name("\\sum"))
    ||space'(" ")
    ||args
    |||text(word'("T"))
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle {\sum T}"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||space'(" ")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||cmd(cmd-name("\\sum"))
    ||||space'(" ")
    ||||text(word'("T"))
    ||||rbrace'("}")
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle [\sum T]"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||space'(" ")
    ||args
    |||bracket
    ||||lbracket'("[")
    ||||cmd(cmd-name("\\sum"))
    ||||space'(" ")
    ||||text(word'("T"))
    ||||rbracket'("]")
    "###);
    assert_debug_snapshot!(parse(r#"T \displaystyle"#), @r###"
    root
    |text(word'("T"),space'(" "))
    |cmd(cmd-name("\\displaystyle"))
    "###);
}

#[test]
fn infix() {
    assert_debug_snapshot!(parse(r#"a \over b'_1"#), @r###"
    root
    |cmd
    ||args
    |||text(word'("a"),space'(" "))
    ||cmd-name("\\over")
    ||args
    |||space'(" ")
    |||attach-comp
    ||||args
    |||||attach-comp
    ||||||args
    |||||||text(word'("b"))
    ||||||apostrophe'("'")
    ||||underline'("_")
    ||||word'("1")
    "###);
    assert_debug_snapshot!(parse(r#"a \over b"#), @r###"
    root
    |cmd
    ||args
    |||text(word'("a"),space'(" "))
    ||cmd-name("\\over")
    ||args
    |||space'(" ")
    |||text(word'("b"))
    "###);
    assert_debug_snapshot!(parse(r#"1 + {2 \over 3}"#), @r###"
    root
    |text(word'("1"),space'(" "),word'("+"),space'(" "))
    |curly
    ||lbrace'("{")
    ||cmd
    |||args
    ||||text(word'("2"),space'(" "))
    |||cmd-name("\\over")
    |||args
    ||||space'(" ")
    ||||text(word'("3"))
    ||rbrace'("}")
    "###);
    // Note: this is an invalid expression
    assert_debug_snapshot!(parse(r#"a \over c \over b"#), @r###"
    root
    |cmd
    ||args
    |||text(word'("a"),space'(" "))
    ||cmd-name("\\over")
    ||args
    |||space'(" ")
    |||text(word'("c"),space'(" "))
    |||cmd
    ||||cmd-name("\\over")
    ||||args
    |||||space'(" ")
    |||||text(word'("b"))
    "###);
}
