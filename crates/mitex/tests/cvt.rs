mod cvt {
    mod prelude {
        pub use insta::assert_snapshot;
        pub use mitex::convert_math as mitex_convert_math;
        pub use mitex::convert_text as mitex_convert_text;
        pub use mitex_spec_gen::DEFAULT_SPEC;

        pub fn convert_text(input: &str) -> Result<String, String> {
            mitex_convert_text(input, Some(DEFAULT_SPEC.clone()))
        }

        pub fn convert_math(input: &str) -> Result<String, String> {
            mitex_convert_math(input, Some(DEFAULT_SPEC.clone()))
        }
    }

    use prelude::*;

    #[cfg(test)]
    mod basic_text_mode;

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
    mod simple_env;

    #[cfg(test)]
    mod trivia;

    #[cfg(test)]
    mod figure;

    #[cfg(test)]
    mod tabular;

    #[cfg(test)]
    mod misc;
    /// Convenient function to launch/debug a test case
    #[test]
    fn bug_playground() {}

    #[test]
    fn test_easy() {
        assert_snapshot!(convert_math(r#"\frac{ a }{ b }"#).unwrap(), @"frac( a  , b  )");
    }

    #[test]
    fn test_utf8_char() {
        // note that there is utf8 minus sign in the middle
        assert_snapshot!(convert_math(r#"$u^−$"#).unwrap(), @"u ^(− )"
        );
    }

    #[test]
    fn test_beat_pandoc() {
        assert_snapshot!(convert_math(r#"\frac 1 2 _3"#).unwrap(), @"frac(1 ,2 ) _(3 )");
    }

    #[test]
    fn test_normal() {
        assert_snapshot!(convert_math(r#"\int_1^2 x \mathrm{d} x"#).unwrap(), @"integral _(1 )^(2 ) x  upright(d ) x ");
    }

    #[test]
    fn test_sticky() {
        assert_snapshot!(convert_math(r#"\alpha_1"#).unwrap(), @"alpha _(1 )");
    }
}
