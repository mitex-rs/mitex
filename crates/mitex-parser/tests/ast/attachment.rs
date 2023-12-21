use super::prelude::*;

#[test]
fn base() {
    // println!("{:#?}", parse(r#"{}_{1}^1"#));
    assert_debug_snapshot!(parse(r#"{}_{1}^2"#), @r###"
    root
    |attach-comp
    ||args
    |||attach-comp
    ||||args
    |||||curly(lbrace'("{"),rbrace'("}"))
    ||||underline'("_")
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("1"))
    |||||rbrace'("}")
    ||caret'("^")
    ||word'("2")
    "###);
    assert_debug_snapshot!(parse(r#"\alpha_1"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd(cmd-name("\\alpha"))
    ||underline'("_")
    ||word'("1")
    "###);
    assert_debug_snapshot!(parse(r#"\alpha_[1]"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd(cmd-name("\\alpha"))
    ||underline'("_")
    ||lbracket'("[")
    |text(word'("1"))
    |rbracket'("]")
    "###);
    assert_debug_snapshot!(parse(r#"\alpha_(1)"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd(cmd-name("\\alpha"))
    ||underline'("_")
    ||lparen'("(")
    |text(word'("1"))
    |rparen'(")")
    "###);
    assert_debug_snapshot!(parse(r#"_1"#), @r###"
    root
    |attach-comp(underline'("_"),word'("1"))
    "###);
    // Note: this is an invalid expression
    assert_debug_snapshot!(parse(r#"\over_1"#), @r###"
    root
    |cmd
    ||args()
    ||cmd-name("\\over")
    ||args
    |||attach-comp(underline'("_"),word'("1"))
    "###);
    assert_debug_snapshot!(parse(r#"{}_1"#), @r###"
    root
    |attach-comp
    ||args
    |||curly(lbrace'("{"),rbrace'("}"))
    ||underline'("_")
    ||word'("1")
    "###);
    assert_debug_snapshot!(parse(r#"{}_1_1"#), @r###"
    root
    |attach-comp
    ||args
    |||attach-comp
    ||||args
    |||||curly(lbrace'("{"),rbrace'("}"))
    ||||underline'("_")
    ||||word'("1")
    ||underline'("_")
    ||word'("1")
    "###);
    assert_debug_snapshot!(parse(r#"\frac{1}{2}_{3}"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd
    ||||cmd-name("\\frac")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("1"))
    ||||||rbrace'("}")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("2"))
    ||||||rbrace'("}")
    ||underline'("_")
    ||curly
    |||lbrace'("{")
    |||text(word'("3"))
    |||rbrace'("}")
    "###);
    assert_debug_snapshot!(parse(r#"\overbrace{a + b + c}^{\text{This is an overbrace}}"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd
    ||||cmd-name("\\overbrace")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("a"),space'(" "),word'("+"),space'(" "),word'("b"),space'(" "),word'("+"),space'(" "),word'("c"))
    ||||||rbrace'("}")
    ||caret'("^")
    ||curly
    |||lbrace'("{")
    |||cmd
    ||||cmd-name("\\text")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("This"),space'(" "),word'("is"),space'(" "),word'("an"),space'(" "),word'("overbrace"))
    ||||||rbrace'("}")
    |||rbrace'("}")
    "###);
    assert_debug_snapshot!(parse(r#"\underbrace{x \times y}_{\text{This is an underbrace}}"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd
    ||||cmd-name("\\underbrace")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("x"),space'(" "))
    ||||||cmd(cmd-name("\\times"))
    ||||||space'(" ")
    ||||||text(word'("y"))
    ||||||rbrace'("}")
    ||underline'("_")
    ||curly
    |||lbrace'("{")
    |||cmd
    ||||cmd-name("\\text")
    ||||args
    |||||curly
    ||||||lbrace'("{")
    ||||||text(word'("This"),space'(" "),word'("is"),space'(" "),word'("an"),space'(" "),word'("underbrace"))
    ||||||rbrace'("}")
    |||rbrace'("}")
    "###);
    assert_debug_snapshot!(parse(r#"x_1''^2"#), @r###"
    root
    |attach-comp
    ||args
    |||attach-comp
    ||||args
    |||||attach-comp
    ||||||args
    |||||||attach-comp
    ||||||||args
    |||||||||text(word'("x"))
    ||||||||underline'("_")
    ||||||||word'("1")
    ||||||apostrophe'("'")
    ||||apostrophe'("'")
    ||caret'("^")
    ||word'("2")
    "###);
    assert_debug_snapshot!(parse(r#"x''_1"#), @r###"
    root
    |attach-comp
    ||args
    |||attach-comp
    ||||args
    |||||attach-comp
    ||||||args
    |||||||text(word'("x"))
    ||||||apostrophe'("'")
    ||||apostrophe'("'")
    ||underline'("_")
    ||word'("1")
    "###);
    assert_debug_snapshot!(parse(r#"''"#), @r###"
    root(apostrophe'("'"),apostrophe'("'"))
    "###);
    assert_debug_snapshot!(parse(r#"\frac''"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||args(apostrophe'("'"))
    ||args(apostrophe'("'"))
    "###);
}

#[test]
fn test_attachment_may_weird() {
    assert_debug_snapshot!(parse(r#"\frac ab_c"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd
    ||||cmd-name("\\frac")
    ||||space'(" ")
    ||||args(word'("a"))
    ||||args(word'("b"))
    ||underline'("_")
    ||word'("c")
    "###);
    assert_debug_snapshot!(parse(r#"\frac a_c b"#), @r###"
    root
    |attach-comp
    ||args
    |||cmd
    ||||cmd-name("\\frac")
    ||||space'(" ")
    ||||args(word'("a"))
    ||underline'("_")
    ||word'("c")
    |space'(" ")
    |text(word'("b"))
    "###);
    assert_debug_snapshot!(parse(r#"\frac {a_c} b"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||space'(" ")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||attach-comp
    |||||args
    ||||||text(word'("a"))
    |||||underline'("_")
    |||||word'("c")
    ||||rbrace'("}")
    ||||space'(" ")
    ||args(word'("b"))
    "###);
}
