use super::prelude::*;

#[test]
fn split_char() {
    assert_debug_snapshot!(parse(r#"\frac abcd"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||space'(" ")
    ||args(word'("a"))
    ||args(word'("b"))
    |text(word'("cd"))
    "###);
    assert_debug_snapshot!(parse(r#"\frac ab"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||space'(" ")
    ||args(word'("a"))
    ||args(word'("b"))
    "###);
    assert_debug_snapshot!(parse(r#"\frac a"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||space'(" ")
    ||args(word'("a"))
    "###);
}

#[test]
fn sqrt_pattern() {
    assert_debug_snapshot!(parse(r#"\sqrt 12"#), @r###"
    root
    |cmd
    ||cmd-name("\\sqrt")
    ||space'(" ")
    ||args(word'("1"))
    |text(word'("2"))
    "###);
    assert_debug_snapshot!(parse(r#"\sqrt{1}2"#), @r###"
    root
    |cmd
    ||cmd-name("\\sqrt")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||text(word'("1"))
    ||||rbrace'("}")
    |text(word'("2"))
    "###);
    // Note: this is an invalid expression
    assert_debug_snapshot!(parse(r#"\sqrt[1]"#), @r###"
    root
    |cmd
    ||cmd-name("\\sqrt")
    ||args
    |||bracket
    ||||lbracket'("[")
    ||||text(word'("1"))
    ||||rbracket'("]")
    "###);
    assert_debug_snapshot!(parse(r#"\sqrt[1]{2}"#), @r###"
    root
    |cmd
    ||cmd-name("\\sqrt")
    ||args
    |||bracket
    ||||lbracket'("[")
    ||||text(word'("1"))
    ||||rbracket'("]")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||text(word'("2"))
    ||||rbrace'("}")
    "###);
    assert_debug_snapshot!(parse(r#"\sqrt[1]{2}3"#), @r###"
    root
    |cmd
    ||cmd-name("\\sqrt")
    ||args
    |||bracket
    ||||lbracket'("[")
    ||||text(word'("1"))
    ||||rbracket'("]")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||text(word'("2"))
    ||||rbrace'("}")
    |text(word'("3"))
    "###);
}
