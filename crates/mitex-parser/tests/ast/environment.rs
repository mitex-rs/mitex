use super::prelude::*;

#[test]
fn matrix() {
    assert_debug_snapshot!(parse(
            r#"\begin{matrix}
a & b \\
c & d
\end{matrix}"#), @r###"
    root
    |env
    ||begin
    |||cmd-name("\\begin")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"))
    ||text(word'("a"),space'(" "))
    ||and'("&")
    ||space'(" ")
    ||text(word'("b"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||text(word'("c"),space'(" "))
    ||and'("&")
    ||space'(" ")
    ||text(word'("d"),br'("\n"))
    ||end
    |||cmd-name("\\end")
    |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"))
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
    |||cmd-name("\\begin")
    |||curly(lbrace'("{"),word'("array"),rbrace'("}"))
    |||args
    ||||curly
    |||||lbrace'("{")
    |||||text(word'("lc"))
    |||||rbrace'("}")
    |||||br'("\n")
    ||text(word'("a"),space'(" "))
    ||and'("&")
    ||space'(" ")
    ||text(word'("b"),space'(" "))
    ||newline("\\\\")
    ||br'("\n")
    ||text(word'("c"),space'(" "))
    ||and'("&")
    ||space'(" ")
    ||text(word'("d"),br'("\n"))
    ||end
    |||cmd-name("\\end")
    |||curly(lbrace'("{"),word'("array"),rbrace'("}"))
    "###);
}
