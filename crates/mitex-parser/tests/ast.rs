pub mod common;

mod ast {
    mod prelude {
        pub use crate::common::parse_snap as parse;
        pub use insta::assert_debug_snapshot;
    }

    use prelude::*;

    #[cfg(test)]
    mod arg_parse;

    #[cfg(test)]
    mod arg_match;

    #[cfg(test)]
    mod attachment;

    #[cfg(test)]
    mod block_comment;

    #[cfg(test)]
    mod formula;

    #[cfg(test)]
    mod fuzzing;

    #[cfg(test)]
    mod command;

    #[cfg(test)]
    mod environment;

    #[cfg(test)]
    mod left_right;

    #[cfg(test)]
    mod trivia;

    #[cfg(test)]
    mod figure;

    #[cfg(test)]
    mod tabular;

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
    fn test_utf8_char() {
        // note that there is utf8 minus sign in the middle
        assert_debug_snapshot!(parse(r#"$u^−$"#), @r###"
        root
        |formula
        ||dollar'("$")
        ||attach-comp
        |||args
        ||||text(word'("u"))
        |||caret'("^")
        |||word'("−")
        ||dollar'("$")
        "###
        );
    }

    #[test]
    fn test_beat_pandoc() {
        assert_debug_snapshot!(parse(r#"\frac 1 2 _3"#), @r###"
        root
        |attach-comp
        ||args
        |||cmd
        ||||cmd-name("\\frac")
        ||||args(word'("1"))
        ||||args(word'("2"))
        |||space'(" ")
        ||underscore'("_")
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
        ||||underscore'("_")
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
        |space'(" ")
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
        ||underscore'("_")
        ||word'("1")
        "###);
    }
}
