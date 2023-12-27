use super::prelude::*;

#[test]
fn curly_group() {
    assert_debug_snapshot!(parse(r#"a \mathbf{strong} text"#), @r###"
    root
    |text(word'("a"),space'(" "))
    |cmd
    ||cmd-name("\\mathbf")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||text(word'("strong"))
    ||||rbrace'("}")
    |space'(" ")
    |text(word'("text"))
    "###);
}

#[test]
fn arguments() {
    assert_debug_snapshot!(parse(r#"\frac { 1 } { 2 }"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||space'(" ")
    ||||text(word'("1"),space'(" "))
    ||||rbrace'("}")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||space'(" ")
    ||||text(word'("2"),space'(" "))
    ||||rbrace'("}")
    "###);
}

#[test]
fn greedy_trivia() {
    assert_debug_snapshot!(parse(r#"a {\displaystyle text } b"#), @r###"
    root
    |text(word'("a"),space'(" "))
    |curly
    ||lbrace'("{")
    ||cmd
    |||cmd-name("\\displaystyle")
    |||args
    ||||space'(" ")
    ||||text(word'("text"),space'(" "))
    ||rbrace'("}")
    |space'(" ")
    |text(word'("b"))
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle text "#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||text(word'("text"),space'(" "))
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle {text} "#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||curly
    ||||lbrace'("{")
    ||||text(word'("text"))
    ||||rbrace'("}")
    |||space'(" ")
    "###);
    assert_debug_snapshot!(parse(r#"\displaystyle {\mathrm {text}} "#), @r###"
    root
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
    |||curly
    ||||lbrace'("{")
    ||||cmd
    |||||cmd-name("\\mathrm")
    |||||args
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("text"))
    |||||||rbrace'("}")
    ||||rbrace'("}")
    |||space'(" ")
    "###);
}
