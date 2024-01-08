use super::prelude::*;

#[test]
fn curly_group() {
    assert_debug_snapshot!(parse(r#"a \textbf{strong} text"#), @r###"
    root
    |text(word'("a"),space'(" "))
    |cmd
    ||cmd-name("\\textbf")
    ||args
    |||curly
    ||||lbrace'("{")
    ||||text(word'("strong"))
    ||||rbrace'("}")
    |space'(" ")
    |text(word'("text"))
    "###);
    assert_debug_snapshot!(parse(r#"x \color {red} yz \frac{1}{2}"#), @r###"
    root
    |text(word'("x"),space'(" "))
    |cmd
    ||cmd-name("\\color")
    ||args
    |||space'(" ")
    |||curly
    ||||lbrace'("{")
    ||||text(word'("red"))
    ||||rbrace'("}")
    |||space'(" ")
    |||text(word'("yz"),space'(" "))
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
    "###);
}

#[test]
fn split_char() {
    assert_debug_snapshot!(parse(r#"\frac abcd"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||args(word'("a"))
    ||args(word'("b"))
    |text(word'("cd"))
    "###);
    assert_debug_snapshot!(parse(r#"\frac ab"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
    ||args(word'("a"))
    ||args(word'("b"))
    "###);
    assert_debug_snapshot!(parse(r#"\frac a"#), @r###"
    root
    |cmd
    ||cmd-name("\\frac")
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
    |space'(" ")
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
    ||begin(sym'("matrix"))
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\displaystyle")
    |||args
    ||||space'(" ")
    ||||text(word'("1"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("2"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||text(word'("3"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("4"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("    ")
    ||end(sym'("matrix"))
    |br'("\n")
    |space'("    ")
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
    ||begin(sym'("matrix"))
    ||br'("\n")
    ||space'("        ")
    ||cmd
    |||cmd-name("\\displaystyle")
    |||args
    ||||space'(" ")
    ||||text(word'("1"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("        ")
    ||text(word'("3"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||space'("    ")
    ||end(sym'("matrix"))
    |br'("\n")
    |space'("    ")
    "###);
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}\frac{1} & {2}\end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin(sym'("matrix"))
    ||cmd
    |||cmd-name("\\frac")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("1"))
    |||||rbrace'("}")
    ||space'(" ")
    ||ampersand'("&")
    ||space'(" ")
    ||curly
    |||lbrace'("{")
    |||text(word'("2"))
    |||rbrace'("}")
    ||end(sym'("matrix"))
    |br'("\n")
    |space'("    ")
    "###);
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}\frac{1} \\ {2}\end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin(sym'("matrix"))
    ||cmd
    |||cmd-name("\\frac")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("1"))
    |||||rbrace'("}")
    |||args(newline("\\\\"))
    ||space'(" ")
    ||curly
    |||lbrace'("{")
    |||text(word'("2"))
    |||rbrace'("}")
    ||end(sym'("matrix"))
    |br'("\n")
    |space'("    ")
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
fn special_marks_in_env() {
    assert_debug_snapshot!(parse(r#"
    \displaystyle \frac{1}{2} \\ \frac{1}{2} 
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |cmd
    ||cmd-name("\\displaystyle")
    ||args
    |||space'(" ")
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
    |||space'(" ")
    |||newline("\\\\")
    |||space'(" ")
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
    |||space'(" ")
    |||br'("\n")
    |||space'("    ")
    "###);
    assert_debug_snapshot!(parse(r#"
    \left. \displaystyle \frac{1}{2} \\ \frac{1}{2} \right.
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |lr
    ||clause-lr(cmd-name("\\left"),word'("."))
    ||space'(" ")
    ||cmd
    |||cmd-name("\\displaystyle")
    |||args
    ||||space'(" ")
    ||||cmd
    |||||cmd-name("\\frac")
    |||||args
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("1"))
    |||||||rbrace'("}")
    |||||args
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("2"))
    |||||||rbrace'("}")
    ||||space'(" ")
    ||||newline("\\\\")
    ||||space'(" ")
    ||||cmd
    |||||cmd-name("\\frac")
    |||||args
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("1"))
    |||||||rbrace'("}")
    |||||args
    ||||||curly
    |||||||lbrace'("{")
    |||||||text(word'("2"))
    |||||||rbrace'("}")
    ||||space'(" ")
    ||clause-lr(cmd-name("\\right"),word'("."))
    |br'("\n")
    |space'("    ")
    "###);
    assert_debug_snapshot!(parse(r#"
    \sqrt[\displaystyle \frac{1}{2} \\ \frac{1}{2} ]{}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |cmd
    ||cmd-name("\\sqrt")
    ||args
    |||bracket
    ||||lbracket'("[")
    ||||cmd
    |||||cmd-name("\\displaystyle")
    |||||args
    ||||||space'(" ")
    ||||||cmd
    |||||||cmd-name("\\frac")
    |||||||args
    ||||||||curly
    |||||||||lbrace'("{")
    |||||||||text(word'("1"))
    |||||||||rbrace'("}")
    |||||||args
    ||||||||curly
    |||||||||lbrace'("{")
    |||||||||text(word'("2"))
    |||||||||rbrace'("}")
    ||||||space'(" ")
    ||||||newline("\\\\")
    ||||||space'(" ")
    ||||||cmd
    |||||||cmd-name("\\frac")
    |||||||args
    ||||||||curly
    |||||||||lbrace'("{")
    |||||||||text(word'("1"))
    |||||||||rbrace'("}")
    |||||||args
    ||||||||curly
    |||||||||lbrace'("{")
    |||||||||text(word'("2"))
    |||||||||rbrace'("}")
    ||||||space'(" ")
    ||||rbracket'("]")
    ||args
    |||curly(lbrace'("{"),rbrace'("}"))
    |br'("\n")
    |space'("    ")
    "###);
    assert_debug_snapshot!(parse(r#"
    \begin{matrix}a \over b \\ c\end{matrix}
    "#), @r###"
    root
    |br'("\n")
    |space'("    ")
    |env
    ||begin(sym'("matrix"))
    ||cmd
    |||args
    ||||text(word'("a"),space'(" "))
    |||cmd-name("\\over")
    |||args
    ||||space'(" ")
    ||||text(word'("b"),space'(" "))
    ||newline("\\\\")
    ||space'(" ")
    ||text(word'("c"))
    ||end(sym'("matrix"))
    |br'("\n")
    |space'("    ")
    "###);
}

#[test]
fn sqrt_pattern() {
    assert_debug_snapshot!(parse(r#"\sqrt 12"#), @r###"
    root
    |cmd
    ||cmd-name("\\sqrt")
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
