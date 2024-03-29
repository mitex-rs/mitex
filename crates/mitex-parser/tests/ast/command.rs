use super::prelude::*;

#[test]
fn test_starrd_command() {
    // Description: If the starred command is defined, it is treated as a starred
    assert_debug_snapshot!(parse(r#"\operatorname*{a}"#), @r###"
    root
    |cmd
    ||cmd-name("\\operatorname*")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||text(word'("a"))
    ||||rbrace'("}")
    "###
    );
    // Description: If the starred command is not defined, it is treated as a normal
    // command
    assert_debug_snapshot!(parse(r#"\varphi*1"#), @r###"
    root
    |cmd(cmd-name("\\varphi"))
    |asterisk'("*")
    |text(word'("1"))
    "###
    );
}

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
    assert_debug_snapshot!(parse(r#"\sum \limits"#), @r###"
    root
    |cmd
    ||args
    |||cmd(cmd-name("\\sum"))
    |||space'(" ")
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
fn greedy_assosiation() {
    // Description: infix greed and right greedy
    assert_debug_snapshot!(parse(r#"1 \over \displaystyle 2"#), @r###"
    root
    |cmd
    ||args
    |||text(word'("1"),space'(" "))
    ||cmd-name("\\over")
    ||args
    |||space'(" ")
    |||cmd
    ||||cmd-name("\\displaystyle")
    ||||args
    |||||space'(" ")
    |||||text(word'("2"))
    "###);
    // Description: right greed and right greedy
    assert_debug_snapshot!(parse(r#"\displaystyle \displaystyle 1 \over 2"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||cmd
    ||||cmd-name("\\displaystyle")
    ||||args
    |||||cmd
    ||||||args
    |||||||space'(" ")
    |||||||text(word'("1"),space'(" "))
    ||||||cmd-name("\\over")
    ||||||args
    |||||||space'(" ")
    |||||||text(word'("2"))
    "###);
    // Description: right greed and infix greedy
    assert_debug_snapshot!(parse(r#"\displaystyle 1 \over 2"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||cmd
    ||||args
    |||||space'(" ")
    |||||text(word'("1"),space'(" "))
    ||||cmd-name("\\over")
    ||||args
    |||||space'(" ")
    |||||text(word'("2"))
    "###);
    // Description: infix greed and infix greedy
    // Note: this is an invalid expression
    assert_debug_snapshot!(parse(r#"a \over c \over b"#), @r###"
    root
    |cmd
    ||args
    |||text(word'("a"),space'(" "))
    ||cmd-name("\\over")
    ||args
    |||cmd
    ||||args
    |||||space'(" ")
    |||||text(word'("c"),space'(" "))
    ||||cmd-name("\\over")
    ||||args
    |||||space'(" ")
    |||||text(word'("b"))
    "###);
}

#[test]
fn right_greedy() {
    // Description: produces an empty argument if the righ side is empty
    assert_debug_snapshot!(parse(r#"\displaystyle"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args()
    "###);
    // Description: correctly works left association
    // left1 commands
    assert_debug_snapshot!(parse(r#"\displaystyle\sum\limits"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||cmd
    ||||args
    |||||cmd(cmd-name("\\sum"))
    ||||cmd-name("\\limits")
    "###);
    // subscript
    assert_debug_snapshot!(parse(r#"\displaystyle x_1"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||attach-comp
    ||||args
    |||||text(word'("x"))
    ||||underscore'("_")
    ||||word'("1")
    "###);
    // prime
    assert_debug_snapshot!(parse(r#"\displaystyle x'"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||attach-comp
    ||||args
    |||||text(word'("x"))
    ||||apostrophe'("'")
    "###);
    // Description: doesn't panic on incorect left association
    // left1 commands
    assert_debug_snapshot!(parse(r#"\displaystyle\limits"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||cmd
    ||||args()
    ||||cmd-name("\\limits")
    "###);
    // subscript
    assert_debug_snapshot!(parse(r#"\displaystyle_1"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||attach-comp(underscore'("_"),word'("1"))
    "###);
    // prime
    assert_debug_snapshot!(parse(r#"\displaystyle'"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args(apostrophe'("'"))
    "###);
    // Description: all right side content is collected to a single argument
    assert_debug_snapshot!(parse(r#"\displaystyle a b c"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||text(word'("a"),space'(" "),word'("b"),space'(" "),word'("c"))
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle \sum T"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||cmd(cmd-name("\\sum"))
    |||space'(" ")
    |||text(word'("T"))
    "###);
    // Curly braces doesn't start a new argument
    assert_debug_snapshot!(parse(r#"\displaystyle{\sum T}"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||cmd(cmd-name("\\sum"))
    ||||space'(" ")
    ||||text(word'("T"))
    ||||rbrace'("}")
    "###);
    // Description: doesn't identify brackets as group
    assert_debug_snapshot!(parse(r#"\displaystyle[\sum T]"#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||lbracket'("[")
    |||cmd(cmd-name("\\sum"))
    |||space'(" ")
    |||text(word'("T"))
    |||rbracket'("]")
    "###);
    // Description: scoped by curly braces
    assert_debug_snapshot!(parse(r#"a + {\displaystyle a b} c"#), @r###"
    root
    |text(word'("a"),space'(" "),word'("+"),space'(" "))
    |curly
    ||lbrace'("{")
    ||cmd
    |||cmd-name("\\displaystyle")
    |||args
    ||||space'(" ")
    ||||text(word'("a"),space'(" "),word'("b"))
    ||rbrace'("}")
    |space'(" ")
    |text(word'("c"))
    "###);
    // Description: doeesn't affect left side
    assert_debug_snapshot!(parse(r#"T \displaystyle"#), @r###"
    root
    |text(word'("T"),space'(" "))
    |cmd
    ||cmd-name("\\displaystyle")
    ||args()
    "###);
}

#[test]
fn infix() {
    assert_debug_snapshot!(parse(r#"\over_1"#), @r###"
    root
    |cmd
    ||args()
    ||cmd-name("\\over")
    ||args
    |||attach-comp(underscore'("_"),word'("1"))
    "###);
    assert_debug_snapshot!(parse(r#"\over'"#), @r###"
    root
    |cmd
    ||args()
    ||cmd-name("\\over")
    ||args(apostrophe'("'"))
    "###);
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
    ||||underscore'("_")
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
}
