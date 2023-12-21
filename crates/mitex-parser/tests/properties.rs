pub mod common;

mod properties {
    use crate::common::parser::parse;
    use insta::assert_debug_snapshot;
    use mitex_parser::syntax::{CmdItem, EnvItem, LRItem, SyntaxKind};
    use rowan::ast::AstNode;

    #[test]
    fn test_env_name() {
        let cmd_node = parse(r#"\begin{equation}\end{equation}"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemEnv))
            .and_then(EnvItem::cast)
            .and_then(|e| e.name_tok())
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###"TokenWord@7..15 "equation""###);
    }

    #[test]
    fn test_cmd_arguments() {
        let cmd_node = parse(r#"\frac ab"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemCmd))
            .and_then(CmdItem::cast)
            .map(|e| e.arguments().collect::<Vec<_>>())
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###"
    [
        ClauseArgument@6..7
          TokenWord@6..7 "a"
        ,
        ClauseArgument@7..8
          TokenWord@7..8 "b"
        ,
    ]
    "###);
        let cmd_node = parse(r#"\displaystyle abcdefg"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemCmd))
            .and_then(CmdItem::cast)
            .map(|e| e.arguments().collect::<Vec<_>>())
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###"
        [
            ClauseArgument@14..21
              ItemText@14..21
                TokenWord@14..21 "abcdefg"
            ,
        ]
        "###);
        let cmd_node = parse(r#"\sum\limits"#)
            .descendants()
            .filter(|node| matches!(node.kind(), SyntaxKind::ItemCmd))
            .flat_map(CmdItem::cast)
            .map(|e| (e.name_tok().unwrap(), e.arguments().collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        assert_debug_snapshot!(cmd_node, @r###"
        [
            (
                ClauseCommandName@4..11 "\\limits",
                [
                    ClauseArgument@0..4
                      ItemCmd@0..4
                        ClauseCommandName@0..4 "\\sum"
                    ,
                ],
            ),
            (
                ClauseCommandName@0..4 "\\sum",
                [],
            ),
        ]
        "###);
    }

    #[test]
    fn test_env_easy() {
        assert_debug_snapshot!(parse(r#"\begin{equation}\end{equation}"#), @r###"
        ScopeRoot@0..30
          ItemEnv@0..30
            ItemBegin@0..16
              ClauseCommandName@0..6 "\\begin"
              ItemCurly@6..16
                TokenLBrace@6..7 "{"
                TokenWord@7..15 "equation"
                TokenRBrace@15..16 "}"
            ItemEnd@16..30
              ClauseCommandName@16..20 "\\end"
              ItemCurly@20..30
                TokenLBrace@20..21 "{"
                TokenWord@21..29 "equation"
                TokenRBrace@29..30 "}"
        "###);
    }

    #[test]
    fn test_lr_item() {
        assert_debug_snapshot!(parse(r#"\left.\right."#), @r###"
      ScopeRoot@0..13
        ItemLR@0..13
          ClauseLR@0..6
            ClauseCommandName@0..5 "\\left"
            TokenWord@5..6 "."
          ClauseLR@6..13
            ClauseCommandName@6..12 "\\right"
            TokenWord@12..13 "."
      "###);
        assert_debug_snapshot!(parse(r#"\left.a\right."#), @r###"
        ScopeRoot@0..14
          ItemLR@0..14
            ClauseLR@0..6
              ClauseCommandName@0..5 "\\left"
              TokenWord@5..6 "."
            ItemText@6..7
              TokenWord@6..7 "a"
            ClauseLR@7..14
              ClauseCommandName@7..13 "\\right"
              TokenWord@13..14 "."
        "###);
        assert_debug_snapshot!(parse(r#"\left.    \right] ,"#), @r###"
        ScopeRoot@0..19
          ItemLR@0..17
            ClauseLR@0..6
              ClauseCommandName@0..5 "\\left"
              TokenWord@5..6 "."
            TokenWhiteSpace@6..10 "    "
            ClauseLR@10..17
              ClauseCommandName@10..16 "\\right"
              TokenRBracket@16..17 "]"
          TokenWhiteSpace@17..18 " "
          ItemText@18..19
            TokenComma@18..19 ","
        "###);
        assert_debug_snapshot!(parse(r#"\left  . a \right    \|"#), @r###"
        ScopeRoot@0..23
          ItemLR@0..23
            ClauseLR@0..8
              ClauseCommandName@0..5 "\\left"
              TokenWhiteSpace@5..7 "  "
              TokenWord@7..8 "."
            TokenWhiteSpace@8..9 " "
            ItemText@9..11
              TokenWord@9..10 "a"
              TokenWhiteSpace@10..11 " "
            ClauseLR@11..23
              ClauseCommandName@11..17 "\\right"
              TokenWhiteSpace@17..21 "    "
              TokenCommandSym@21..23 "\\|"
        "###);
        assert_debug_snapshot!(parse(r#"\left\langle a\right\|"#), @r###"
        ScopeRoot@0..22
          ItemLR@0..22
            ClauseLR@0..12
              ClauseCommandName@0..5 "\\left"
              TokenCommandSym@5..12 "\\langle"
            TokenWhiteSpace@12..13 " "
            ItemText@13..14
              TokenWord@13..14 "a"
            ClauseLR@14..22
              ClauseCommandName@14..20 "\\right"
              TokenCommandSym@20..22 "\\|"
        "###);
        // Note: this is an invalid expression
        // Error handling
        assert_debug_snapshot!(parse(r#"\left{.}a\right{.}"#), @r###"
        ScopeRoot@0..18
          ItemLR@0..7
            ClauseLR@0..6
              ClauseCommandName@0..5 "\\left"
              TokenLBrace@5..6 "{"
            ItemText@6..7
              TokenWord@6..7 "."
          TokenError@7..8
            TokenRBrace@7..8 "}"
          ItemText@8..9
            TokenWord@8..9 "a"
          ItemCmd@9..15
            ClauseCommandName@9..15 "\\right"
          ItemCurly@15..18
            TokenLBrace@15..16 "{"
            ItemText@16..17
              TokenWord@16..17 "."
            TokenRBrace@17..18 "}"
        "###);
        // Note: this is an invalid expression
        // Error handling
        assert_debug_snapshot!(parse(r#"\begin{equation}\left.\right\end{equation}"#), @r###"
        ScopeRoot@0..42
          ItemEnv@0..42
            ItemBegin@0..16
              ClauseCommandName@0..6 "\\begin"
              ItemCurly@6..16
                TokenLBrace@6..7 "{"
                TokenWord@7..15 "equation"
                TokenRBrace@15..16 "}"
            ItemLR@16..28
              ClauseLR@16..22
                ClauseCommandName@16..21 "\\left"
                TokenWord@21..22 "."
              ClauseLR@22..28
                ClauseCommandName@22..28 "\\right"
            ItemEnd@28..42
              ClauseCommandName@28..32 "\\end"
              ItemCurly@32..42
                TokenLBrace@32..33 "{"
                TokenWord@33..41 "equation"
                TokenRBrace@41..42 "}"
        "###);
        // Note: this is an invalid expression
        // Error handling
        assert_debug_snapshot!(parse(r#"\begin{equation}\left\right\end{equation}"#), @r###"
        ScopeRoot@0..41
          ItemEnv@0..41
            ItemBegin@0..16
              ClauseCommandName@0..6 "\\begin"
              ItemCurly@6..16
                TokenLBrace@6..7 "{"
                TokenWord@7..15 "equation"
                TokenRBrace@15..16 "}"
            ItemLR@16..27
              ClauseLR@16..21
                ClauseCommandName@16..21 "\\left"
              ClauseLR@21..27
                ClauseCommandName@21..27 "\\right"
            ItemEnd@27..41
              ClauseCommandName@27..31 "\\end"
              ItemCurly@31..41
                TokenLBrace@31..32 "{"
                TokenWord@32..40 "equation"
                TokenRBrace@40..41 "}"
        "###);
    }

    #[test]
    fn test_lr_symbol() {
        fn lr_info(e: LRItem) -> String {
            format!(
                "{:?}",
                (
                    e.left().map(|l| l.is_left()),
                    e.left_sym(),
                    e.right().map(|l| l.is_left()),
                    e.right_sym(),
                )
            )
        }

        let cmd_node = parse(r#"\left.\right."#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), Some(TokenWord@5..6 \".\"), Some(false), Some(TokenWord@12..13 \".\"))""###);
        let cmd_node = parse(r#"\left(\right)"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), Some(TokenLParen@5..6 \"(\"), Some(false), Some(TokenRParen@12..13 \")\"))""###);
        let cmd_node = parse(r#"\left"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), None, Some(true), None)""###);
        // Note: this is an invalid expression, and produce an expected error
        let cmd_node = parse(r#"\left."#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), Some(TokenWord@5..6 \".\"), Some(true), Some(TokenWord@5..6 \".\"))""###);
        let cmd_node = parse(r#"\left\right"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), None, Some(false), None)""###);
        let cmd_node = parse(r#"\left . a\right ."#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), Some(TokenWord@6..7 \".\"), Some(false), Some(TokenWord@16..17 \".\"))""###);
        let cmd_node = parse(r#"\left\langle a\right\|"#)
            .descendants()
            .find(|node| matches!(node.kind(), SyntaxKind::ItemLR))
            .and_then(LRItem::cast)
            .map(lr_info)
            .unwrap();
        assert_debug_snapshot!(cmd_node, @r###""(Some(true), Some(TokenCommandSym@5..12 \"\\\\langle\"), Some(false), Some(TokenCommandSym@20..22 \"\\\\|\"))""###);
    }
}
