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
fn eat_regular_brace() {
    assert_debug_snapshot!(parse(r#"\mathrm(x)"#), @r###"
    root
    |cmd
    ||cmd-name("\\mathrm")
    ||args(lparen'("("))
    |text(word'("x"))
    |rparen'(")")
    "###);
    assert_debug_snapshot!(parse(r#"\mathrm[x]"#), @r###"
    root
    |cmd
    ||cmd-name("\\mathrm")
    ||args(lbracket'("["))
    |text(word'("x"))
    |rbracket'("]")
    "###);
    assert_debug_snapshot!(parse(r#"\mathrm\lbrace x \rbrace"#), @r###"
    root
    |cmd
    ||cmd-name("\\mathrm")
    ||args
    |||cmd(cmd-name("\\lbrace"))
    ||space'(" ")
    |text(word'("x"),space'(" "))
    |cmd(cmd-name("\\rbrace"))
    "###);
}

#[test]
fn special_marks() {
    // & and newline'
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}
        \displaystyle 1 & 2 \\
        3 & 4 \\
    \end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||cmd-name("\\begin")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("        "))
    ||cmd
    |||cmd-name("\\displaystyle")
    |||space'(" ")
    |||args
    ||||text(word'("1"),space'(" "))
    ||and'("&")
    ||space'(" ")
    ||text(word'("2"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||text(word'("3"),space'(" "))
    ||and'("&")
    ||space'(" ")
    ||text(word'("4"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("    ")
    ||end
    |||cmd-name("\\end")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("    "))
    "###);
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}
        \displaystyle 1 \\
        3 \\
    \end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||cmd-name("\\begin")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("        "))
    ||cmd
    |||cmd-name("\\displaystyle")
    |||space'(" ")
    |||args
    ||||text(word'("1"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||text(word'("3"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("    ")
    ||end
    |||cmd-name("\\end")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("    "))
    "###);
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}\frac{1} & {2}\end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||cmd-name("\\begin")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"))
    ||cmd
    |||cmd-name("\\frac")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("1"))
    |||||rbrace'("}")
    |||||space'(" ")
    ||and'("&")
    ||space'(" ")
    ||curly
    |||lbrace'("{")
    |||text(word'("2"))
    |||rbrace'("}")
    ||end
    |||cmd-name("\\end")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("    "))
    "###);
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}\frac{1} \\ {2}\end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin
    |||cmd-name("\\begin")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"))
    ||cmd
    |||cmd-name("\\frac")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("1"))
    |||||rbrace'("}")
    |||||space'(" ")
    |||args(newline("\\\\"))
    |||space'(" ")
    ||curly
    |||lbrace'("{")
    |||text(word'("2"))
    |||rbrace'("}")
    ||end
    |||cmd-name("\\end")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("    "))
    "###);
    assert_debug_snapshot!(parse(r#"
    1 \over 2 \\ 3 
    "#), @r###"
    root
    |cmd
    ||args
    |||br'("\n")
    |||space'("    ")
    |||text(word'("1"),space'(" "))
    ||cmd-name("\\over")
    ||args
    |||space'(" ")
    |||text(word'("2"),space'(" "))
    |||newline("\\\\")
    |||space'(" ")
    |||text(word'("3"),space'(" "),br'("\n"),space'("    "))
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
