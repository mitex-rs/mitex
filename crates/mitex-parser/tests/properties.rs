pub mod common;

mod properties {
    use crate::common::*;
    use insta::{assert_debug_snapshot, assert_snapshot};
    use mitex_parser::syntax::{CmdItem, EnvItem, FormulaItem, LRItem, SyntaxKind, SyntaxToken};
    use rowan::ast::AstNode;

    #[test]
    fn test_env_name() {
        fn env_name(input: &str) -> Option<SnapToken> {
            parse(input)
                .descendants()
                .find(|node| matches!(node.kind(), SyntaxKind::ItemEnv))
                .and_then(EnvItem::cast)
                .and_then(|e| e.name_tok())
                .map(SnapToken)
        }

        assert_debug_snapshot!(env_name(r#"\begin{equation}\end{equation}"#).unwrap(), @r###"sym'("equation")"###);
    }

    #[test]
    fn test_formula_display() {
        fn formula_item(input: &str) -> Option<FormulaItem> {
            parse(input)
                .descendants()
                .find(|node| matches!(node.kind(), SyntaxKind::ItemFormula))
                .and_then(FormulaItem::cast)
        }

        assert!(formula_item(r#"$$a$$"#).unwrap().is_display());
        assert!(!formula_item(r#"$$a$$"#).unwrap().is_inline());
        assert!(!formula_item(r#"$a$"#).unwrap().is_display());
        assert!(formula_item(r#"$a$"#).unwrap().is_inline());
    }

    #[test]
    fn test_cmd_arguments() {
        fn cmd_args(input: &str) -> String {
            parse(input)
                .descendants()
                .filter(|node| matches!(node.kind(), SyntaxKind::ItemCmd))
                .filter_map(CmdItem::cast)
                .map(|e| {
                    let args = e
                        .arguments()
                        .map(SnapNode)
                        .map(|e| format!("{e:#?}").trim().to_string())
                        .collect::<Vec<_>>()
                        .join("\n---\n");

                    format!(
                        "name: {:#?}\n{}",
                        e.name_tok().map(SnapToken).unwrap(),
                        args
                    )
                })
                .collect::<Vec<_>>()
                .join("\n----\n")
        }

        assert_snapshot!(cmd_args(r#"\frac ab"#), @r###"
        name: cmd-name("\\frac")
        args(word'("a"))
        ---
        args(word'("b"))
        "###);
        assert_snapshot!(cmd_args(r#"\displaystyle abcdefg"#), @r###"
        name: cmd-name("\\displaystyle")
        args
        |space'(" ")
        |text(word'("abcdefg"))
        "###);
        assert_snapshot!(cmd_args(r#"\sum\limits"#), @r###"
        name: cmd-name("\\limits")
        args
        |cmd(cmd-name("\\sum"))
        ----
        name: cmd-name("\\sum")
        "###);
    }

    #[test]
    fn test_lr_symbol() {
        fn lr_info(input: &str) -> Option<String> {
            let e = parse(input)
                .descendants()
                .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
                .and_then(LRItem::cast)?;

            fn pretty_sym(s: Option<SyntaxToken>) -> String {
                s.map(SnapToken)
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "None".to_string())
            }

            Some(
                [
                    format!("{:?}", e.left().map(|l| l.is_left())),
                    pretty_sym(e.left_sym()),
                    format!("{:?}", e.right().map(|l| l.is_left())),
                    pretty_sym(e.right_sym()),
                ]
                .join(", "),
            )
        }

        assert_snapshot!(lr_info(r#"\left.\right."#).unwrap(), @r###"Some(true), word'("."), Some(false), word'(".")"###);
        assert_snapshot!(lr_info(r#"\left(\right)"#).unwrap(), @r###"Some(true), lparen'("("), Some(false), rparen'(")")"###);

        assert_snapshot!(lr_info(r#"\left"#).unwrap(), @"Some(true), None, Some(true), None");
        // Note: this is an invalid expression, and produce an expected error
        assert_snapshot!(lr_info(r#"\left."#).unwrap(), @r###"Some(true), word'("."), Some(true), word'(".")"###);

        assert_snapshot!(lr_info(r#"\left\right"#).unwrap(), @"Some(true), None, Some(false), None");
        assert_snapshot!(lr_info(r#"\left . a\right ."#).unwrap(), @r###"Some(true), word'("."), Some(false), word'(".")"###);
        assert_snapshot!(lr_info(r#"\left\langle a\right\|"#).unwrap(), @r###"Some(true), sym'("\\langle"), Some(false), sym'("\\|")"###);
    }
}
