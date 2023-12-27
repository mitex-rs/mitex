use super::prelude::*;

#[test]
fn easy() {
    assert_debug_snapshot!(parse(r#"\begin{equation}\end{equation}"#), @r###"
    root
    |env
    ||begin(sym'("equation"))
    ||end(sym'("equation"))
    "###);
}

#[test]
fn matrix() {
    assert_debug_snapshot!(parse(
            r#"\begin{matrix}
a & b \\
c & d
\end{matrix}"#), @r###"
    root
    |env
    ||begin(sym'("matrix"))
    ||br'("\n")
    ||text(word'("a"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("b"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||text(word'("c"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("d"),br'("\n"))
    ||end(sym'("matrix"))
    "###);
    assert_debug_snapshot!(parse(
            r#"\begin{pmatrix}\\\end{pmatrix}"#), @r###"
    root
    |env
    ||begin(sym'("pmatrix"))
    ||newline("\\\\")
    ||end(sym'("pmatrix"))
    "###);
    assert_debug_snapshot!(parse(
            r#"\begin{pmatrix}x{\\}x\end{pmatrix}"#), @r###"
    root
    |env
    ||begin(sym'("pmatrix"))
    ||text(word'("x"))
    ||curly(lbrace'("{"),newline("\\\\"),rbrace'("}"))
    ||text(word'("x"))
    ||end(sym'("pmatrix"))
    "###);
}

#[test]
fn arguments() {
    assert_debug_snapshot!(parse(
            r#"\begin{array}{lc}
a & b \\
c & d
\end{array}"#), @r###"
    root
    |env
    ||begin
    |||sym'("array")
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("lc"))
    |||||rbrace'("}")
    ||br'("\n")
    ||text(word'("a"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("b"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||text(word'("c"),space'(" "))
    ||ampersand'("&")
    ||space'(" ")
    ||text(word'("d"),br'("\n"))
    ||end(sym'("array"))
    "###);
}

#[test]
fn space_around_and() {
    assert_debug_snapshot!(parse(
            r#"\begin{bmatrix}A&B\end{bmatrix}"#), @r###"
    root
    |env
    ||begin(sym'("bmatrix"))
    ||text(word'("A"))
    ||ampersand'("&")
    ||text(word'("B"))
    ||end(sym'("bmatrix"))
    "###);
}
