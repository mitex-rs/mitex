pub mod common;

mod ast {
    use insta::assert_debug_snapshot;

    use crate::common::parse_snap as parse;

    // #[cfg(test)]
    // mod frac;

    /// Convenient function to launch/debug a test case
    #[test]
    fn bug_playground() {}

    #[test]
    fn test_easy() {
        assert_debug_snapshot!(parse(r#"\frac{ a }{ b }"#), @r###"
        root
        |cmd
        ||cmd-name("\\frac")
        ||args
        |||curly
        ||||lbrace'("{")
        ||||space'(" ")
        ||||text(word'("a"),space'(" "))
        ||||rbrace'("}")
        ||args
        |||curly
        ||||lbrace'("{")
        ||||space'(" ")
        ||||text(word'("b"),space'(" "))
        ||||rbrace'("}")
        "###);
    }

    #[test]
    fn test_beat_pandoc() {
        assert_debug_snapshot!(parse(r#"\frac 1 2 _3"#), @r###"
        root
        |attach-comp
        ||args
        |||cmd
        ||||cmd-name("\\frac")
        ||||space'(" ")
        ||||args(word'("1"))
        ||||space'(" ")
        ||||args(word'("2"))
        ||||space'(" ")
        ||underline'("_")
        ||word'("3")
        "###);
    }

    #[test]
    fn test_normal() {
        assert_debug_snapshot!(parse(r#"\int_1^2 x \mathrm{d} x"#), @r###"
        root
        |attach-comp
        ||args
        |||attach-comp
        ||||args
        |||||cmd(cmd-name("\\int"))
        ||||underline'("_")
        ||||word'("1")
        ||caret'("^")
        ||word'("2")
        |space'(" ")
        |text(word'("x"),space'(" "))
        |cmd
        ||cmd-name("\\mathrm")
        ||args
        |||curly
        ||||lbrace'("{")
        ||||text(word'("d"))
        ||||rbrace'("}")
        ||||space'(" ")
        |text(word'("x"))
        "###);
    }

    #[test]
    fn test_sticky() {
        assert_debug_snapshot!(parse(r#"\alpha_1"#), @r###"
        root
        |attach-comp
        ||args
        |||cmd(cmd-name("\\alpha"))
        ||underline'("_")
        ||word'("1")
        "###);
    }

    #[test]
    fn test_cmd_split() {
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
    fn test_cmd_left_association() {
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
    fn test_cmd_right_greedy() {
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
    fn test_cmd_infix() {
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

    // #[test]
    // fn test_sqrt() {
    //     assert_debug_snapshot!(parse(r#"\sqrt a"#), @r###""###);
    // }

    #[test]
    fn test_env_matrix() {
        assert_debug_snapshot!(parse(
                r#"\begin{matrix}
  a & b \\
  c & d
\end{matrix}"#), @r###"
        root
        |env
        ||begin
        |||cmd-name("\\begin")
        |||curly(lbrace'("{"),word'("matrix"),rbrace'("}"),br'("\n"),space'("  "))
        ||text(word'("a"),space'(" "))
        ||and'("&")
        ||space'(" ")
        ||text(word'("b"),space'(" "))
        ||newline("\\\\")
        ||br'("\n")
        ||space'("  ")
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
    fn test_env_with_options() {
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
        |||||space'("  ")
        ||text(word'("a"),space'(" "))
        ||and'("&")
        ||space'(" ")
        ||text(word'("b"),space'(" "))
        ||newline("\\\\")
        ||br'("\n")
        ||space'("  ")
        ||text(word'("c"),space'(" "))
        ||and'("&")
        ||space'(" ")
        ||text(word'("d"),br'("\n"))
        ||end
        |||cmd-name("\\end")
        |||curly(lbrace'("{"),word'("array"),rbrace'("}"))
        "###);
    }

    #[test]
    fn test_attachment() {
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

    #[test]
    fn test_sqrt() {
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
}
