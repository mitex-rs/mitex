use super::prelude::*;

#[test]
fn test_fuzzing() {
    assert_snapshot!(convert_math(r#"\left\0"#).unwrap_err(), @r###"error: unknown command: \0"###);
    assert_snapshot!(convert_math(r#"\end{}"#).unwrap_err(), @r###"error: error unexpected: """###);
}

#[test]
fn unbraced_tabular_argument() {
    assert_snapshot!(convert_math(r"\begin{tabular} a \end{tabular}").unwrap_err(), @"error: tabular column spec must be braced");
}

#[test]
fn newcommand_naming_an_environment() {
    assert!(convert_math(r"\newcommand{\begin{matrix}_").is_ok());
}
