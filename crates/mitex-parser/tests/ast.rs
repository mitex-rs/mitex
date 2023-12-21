pub mod common;

mod ast {
    use insta::assert_debug_snapshot;

    // use crate::common::parse_snap as parse;
    use crate::common::parse;

    // #[cfg(test)]
    // mod frac;

    /// Convenient function to launch/debug a test case
    #[test]
    fn bug_playground() {}

    #[test]
    fn test_easy() {
        assert_debug_snapshot!(parse(r#"\frac{ a }{ b }"#), @r###"
        ScopeRoot@0..15
          ItemCmd@0..15
            ClauseCommandName@0..5 "\\frac"
            ClauseArgument@5..10
              ItemCurly@5..10
                TokenLBrace@5..6 "{"
                TokenWhiteSpace@6..7 " "
                ItemText@7..9
                  TokenWord@7..8 "a"
                  TokenWhiteSpace@8..9 " "
                TokenRBrace@9..10 "}"
            ClauseArgument@10..15
              ItemCurly@10..15
                TokenLBrace@10..11 "{"
                TokenWhiteSpace@11..12 " "
                ItemText@12..14
                  TokenWord@12..13 "b"
                  TokenWhiteSpace@13..14 " "
                TokenRBrace@14..15 "}"
        "###);
    }

    #[test]
    fn test_beat_pandoc() {
        assert_debug_snapshot!(parse(r#"\frac 1 2 _3"#), @r###"
        ScopeRoot@0..12
          ItemAttachComponent@0..12
            ClauseArgument@0..10
              ItemCmd@0..10
                ClauseCommandName@0..5 "\\frac"
                TokenWhiteSpace@5..6 " "
                ClauseArgument@6..7
                  TokenWord@6..7 "1"
                TokenWhiteSpace@7..8 " "
                ClauseArgument@8..9
                  TokenWord@8..9 "2"
                TokenWhiteSpace@9..10 " "
            TokenUnderline@10..11 "_"
            TokenWord@11..12 "3"
        "###);
    }

    #[test]
    fn test_normal() {
        assert_debug_snapshot!(parse(r#"\int_1^2 x \mathrm{d} x"#), @r###"
        ScopeRoot@0..23
          ItemAttachComponent@0..8
            ClauseArgument@0..6
              ItemAttachComponent@0..6
                ClauseArgument@0..4
                  ItemCmd@0..4
                    ClauseCommandName@0..4 "\\int"
                TokenUnderline@4..5 "_"
                TokenWord@5..6 "1"
            TokenCaret@6..7 "^"
            TokenWord@7..8 "2"
          TokenWhiteSpace@8..9 " "
          ItemText@9..11
            TokenWord@9..10 "x"
            TokenWhiteSpace@10..11 " "
          ItemCmd@11..22
            ClauseCommandName@11..18 "\\mathrm"
            ClauseArgument@18..22
              ItemCurly@18..22
                TokenLBrace@18..19 "{"
                ItemText@19..20
                  TokenWord@19..20 "d"
                TokenRBrace@20..21 "}"
                TokenWhiteSpace@21..22 " "
          ItemText@22..23
            TokenWord@22..23 "x"
        "###);
    }

    #[test]
    fn test_sticky() {
        assert_debug_snapshot!(parse(r#"\alpha_1"#), @r###"
        ScopeRoot@0..8
          ItemAttachComponent@0..8
            ClauseArgument@0..6
              ItemCmd@0..6
                ClauseCommandName@0..6 "\\alpha"
            TokenUnderline@6..7 "_"
            TokenWord@7..8 "1"
        "###);
    }

    #[test]
    fn test_cmd_split() {
        assert_debug_snapshot!(parse(r#"\frac abcd"#), @r###"
        ScopeRoot@0..10
          ItemCmd@0..8
            ClauseCommandName@0..5 "\\frac"
            TokenWhiteSpace@5..6 " "
            ClauseArgument@6..7
              TokenWord@6..7 "a"
            ClauseArgument@7..8
              TokenWord@7..8 "b"
          ItemText@8..10
            TokenWord@8..10 "cd"
        "###);
        assert_debug_snapshot!(parse(r#"\frac ab"#), @r###"
        ScopeRoot@0..8
          ItemCmd@0..8
            ClauseCommandName@0..5 "\\frac"
            TokenWhiteSpace@5..6 " "
            ClauseArgument@6..7
              TokenWord@6..7 "a"
            ClauseArgument@7..8
              TokenWord@7..8 "b"
        "###);
        assert_debug_snapshot!(parse(r#"\frac a"#), @r###"
        ScopeRoot@0..7
          ItemCmd@0..7
            ClauseCommandName@0..5 "\\frac"
            TokenWhiteSpace@5..6 " "
            ClauseArgument@6..7
              TokenWord@6..7 "a"
        "###);
    }

    #[test]
    fn test_cmd_left_association() {
        assert_debug_snapshot!(parse(r#"\sum"#), @r###"
        ScopeRoot@0..4
          ItemCmd@0..4
            ClauseCommandName@0..4 "\\sum"
        "###);
        assert_debug_snapshot!(parse(r#"\sum\limits"#), @r###"
        ScopeRoot@0..11
          ItemCmd@0..11
            ClauseArgument@0..4
              ItemCmd@0..4
                ClauseCommandName@0..4 "\\sum"
            ClauseCommandName@4..11 "\\limits"
        "###);
        assert_debug_snapshot!(parse(r#"\sum\limits\limits"#), @r###"
        ScopeRoot@0..18
          ItemCmd@0..18
            ClauseArgument@0..11
              ItemCmd@0..11
                ClauseArgument@0..4
                  ItemCmd@0..4
                    ClauseCommandName@0..4 "\\sum"
                ClauseCommandName@4..11 "\\limits"
            ClauseCommandName@11..18 "\\limits"
        "###);
        assert_debug_snapshot!(parse(r#"\sum\limits\sum"#), @r###"
        ScopeRoot@0..15
          ItemCmd@0..11
            ClauseArgument@0..4
              ItemCmd@0..4
                ClauseCommandName@0..4 "\\sum"
            ClauseCommandName@4..11 "\\limits"
          ItemCmd@11..15
            ClauseCommandName@11..15 "\\sum"
        "###);
        assert_debug_snapshot!(parse(r#"\sum\limits\sum\limits"#), @r###"
        ScopeRoot@0..22
          ItemCmd@0..11
            ClauseArgument@0..4
              ItemCmd@0..4
                ClauseCommandName@0..4 "\\sum"
            ClauseCommandName@4..11 "\\limits"
          ItemCmd@11..22
            ClauseArgument@11..15
              ItemCmd@11..15
                ClauseCommandName@11..15 "\\sum"
            ClauseCommandName@15..22 "\\limits"
        "###);
        assert_debug_snapshot!(parse(r#"\limits"#), @r###"
        ScopeRoot@0..7
          ItemCmd@0..7
            ClauseArgument@0..0
            ClauseCommandName@0..7 "\\limits"
        "###);
    }

    #[test]
    fn test_cmd_right_greedy() {
        assert_debug_snapshot!(parse(r#"\displaystyle"#), @r###"
        ScopeRoot@0..13
          ItemCmd@0..13
            ClauseCommandName@0..13 "\\displaystyle"
        "###);
        assert_debug_snapshot!(parse(r#"\displaystyle a b c"#), @r###"
        ScopeRoot@0..19
          ItemCmd@0..19
            ClauseCommandName@0..13 "\\displaystyle"
            TokenWhiteSpace@13..14 " "
            ClauseArgument@14..19
              ItemText@14..19
                TokenWord@14..15 "a"
                TokenWhiteSpace@15..16 " "
                TokenWord@16..17 "b"
                TokenWhiteSpace@17..18 " "
                TokenWord@18..19 "c"
        "###);
        assert_debug_snapshot!(parse(r#"a + {\displaystyle a b} c"#), @r###"
        ScopeRoot@0..25
          ItemText@0..4
            TokenWord@0..1 "a"
            TokenWhiteSpace@1..2 " "
            TokenWord@2..3 "+"
            TokenWhiteSpace@3..4 " "
          ItemCurly@4..24
            TokenLBrace@4..5 "{"
            ItemCmd@5..22
              ClauseCommandName@5..18 "\\displaystyle"
              TokenWhiteSpace@18..19 " "
              ClauseArgument@19..22
                ItemText@19..22
                  TokenWord@19..20 "a"
                  TokenWhiteSpace@20..21 " "
                  TokenWord@21..22 "b"
            TokenRBrace@22..23 "}"
            TokenWhiteSpace@23..24 " "
          ItemText@24..25
            TokenWord@24..25 "c"
        "###);
        assert_debug_snapshot!(parse(r#"\displaystyle \sum T"#), @r###"
        ScopeRoot@0..20
          ItemCmd@0..20
            ClauseCommandName@0..13 "\\displaystyle"
            TokenWhiteSpace@13..14 " "
            ClauseArgument@14..18
              ItemCmd@14..18
                ClauseCommandName@14..18 "\\sum"
            TokenWhiteSpace@18..19 " "
            ClauseArgument@19..20
              ItemText@19..20
                TokenWord@19..20 "T"
        "###);
        assert_debug_snapshot!(parse(r#"\displaystyle {\sum T}"#), @r###"
        ScopeRoot@0..22
          ItemCmd@0..22
            ClauseCommandName@0..13 "\\displaystyle"
            TokenWhiteSpace@13..14 " "
            ClauseArgument@14..22
              ItemCurly@14..22
                TokenLBrace@14..15 "{"
                ItemCmd@15..19
                  ClauseCommandName@15..19 "\\sum"
                TokenWhiteSpace@19..20 " "
                ItemText@20..21
                  TokenWord@20..21 "T"
                TokenRBrace@21..22 "}"
        "###);
        assert_debug_snapshot!(parse(r#"\displaystyle [\sum T]"#), @r###"
        ScopeRoot@0..22
          ItemCmd@0..22
            ClauseCommandName@0..13 "\\displaystyle"
            TokenWhiteSpace@13..14 " "
            ClauseArgument@14..22
              ItemBracket@14..22
                TokenLBracket@14..15 "["
                ItemCmd@15..19
                  ClauseCommandName@15..19 "\\sum"
                TokenWhiteSpace@19..20 " "
                ItemText@20..21
                  TokenWord@20..21 "T"
                TokenRBracket@21..22 "]"
        "###);
        assert_debug_snapshot!(parse(r#"T \displaystyle"#), @r###"
        ScopeRoot@0..15
          ItemText@0..2
            TokenWord@0..1 "T"
            TokenWhiteSpace@1..2 " "
          ItemCmd@2..15
            ClauseCommandName@2..15 "\\displaystyle"
        "###);
    }

    #[test]
    fn test_cmd_infix() {
        assert_debug_snapshot!(parse(r#"a \over b'_1"#), @r###"
      ScopeRoot@0..12
        ItemCmd@0..12
          ClauseArgument@0..2
            ItemText@0..2
              TokenWord@0..1 "a"
              TokenWhiteSpace@1..2 " "
          ClauseCommandName@2..7 "\\over"
          ClauseArgument@7..12
            TokenWhiteSpace@7..8 " "
            ItemAttachComponent@8..12
              ClauseArgument@8..10
                ItemAttachComponent@8..10
                  ClauseArgument@8..9
                    ItemText@8..9
                      TokenWord@8..9 "b"
                  TokenApostrophe@9..10 "'"
              TokenUnderline@10..11 "_"
              TokenWord@11..12 "1"
      "###);
        assert_debug_snapshot!(parse(r#"a \over b"#), @r###"
        ScopeRoot@0..9
          ItemCmd@0..9
            ClauseArgument@0..2
              ItemText@0..2
                TokenWord@0..1 "a"
                TokenWhiteSpace@1..2 " "
            ClauseCommandName@2..7 "\\over"
            ClauseArgument@7..9
              TokenWhiteSpace@7..8 " "
              ItemText@8..9
                TokenWord@8..9 "b"
        "###);
        assert_debug_snapshot!(parse(r#"1 + {2 \over 3}"#), @r###"
        ScopeRoot@0..15
          ItemText@0..4
            TokenWord@0..1 "1"
            TokenWhiteSpace@1..2 " "
            TokenWord@2..3 "+"
            TokenWhiteSpace@3..4 " "
          ItemCurly@4..15
            TokenLBrace@4..5 "{"
            ItemCmd@5..14
              ClauseArgument@5..7
                ItemText@5..7
                  TokenWord@5..6 "2"
                  TokenWhiteSpace@6..7 " "
              ClauseCommandName@7..12 "\\over"
              ClauseArgument@12..14
                TokenWhiteSpace@12..13 " "
                ItemText@13..14
                  TokenWord@13..14 "3"
            TokenRBrace@14..15 "}"
        "###);
        // Note: this is an invalid expression
        assert_debug_snapshot!(parse(r#"a \over c \over b"#), @r###"
        ScopeRoot@0..17
          ItemCmd@0..17
            ClauseArgument@0..2
              ItemText@0..2
                TokenWord@0..1 "a"
                TokenWhiteSpace@1..2 " "
            ClauseCommandName@2..7 "\\over"
            ClauseArgument@7..17
              TokenWhiteSpace@7..8 " "
              ItemText@8..10
                TokenWord@8..9 "c"
                TokenWhiteSpace@9..10 " "
              ItemCmd@10..17
                ClauseCommandName@10..15 "\\over"
                ClauseArgument@15..17
                  TokenWhiteSpace@15..16 " "
                  ItemText@16..17
                    TokenWord@16..17 "b"
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
        ScopeRoot@0..46
          ItemEnv@0..46
            ItemBegin@0..17
              ClauseCommandName@0..6 "\\begin"
              ItemCurly@6..17
                TokenLBrace@6..7 "{"
                TokenWord@7..13 "matrix"
                TokenRBrace@13..14 "}"
                TokenLineBreak@14..15 "\n"
                TokenWhiteSpace@15..17 "  "
            ItemText@17..19
              TokenWord@17..18 "a"
              TokenWhiteSpace@18..19 " "
            TokenAnd@19..20 "&"
            TokenWhiteSpace@20..21 " "
            ItemText@21..23
              TokenWord@21..22 "b"
              TokenWhiteSpace@22..23 " "
            ItemNewLine@23..25 "\\\\"
            TokenLineBreak@25..26 "\n"
            TokenWhiteSpace@26..28 "  "
            ItemText@28..30
              TokenWord@28..29 "c"
              TokenWhiteSpace@29..30 " "
            TokenAnd@30..31 "&"
            TokenWhiteSpace@31..32 " "
            ItemText@32..34
              TokenWord@32..33 "d"
              TokenLineBreak@33..34 "\n"
            ItemEnd@34..46
              ClauseCommandName@34..38 "\\end"
              ItemCurly@38..46
                TokenLBrace@38..39 "{"
                TokenWord@39..45 "matrix"
                TokenRBrace@45..46 "}"
        "###);
    }

    #[test]
    fn test_env_with_options() {
        assert_debug_snapshot!(parse(
                r#"\begin{array}{lc}
  a & b \\
  c & d
\end{array}"#), @r###"
        ScopeRoot@0..48
          ItemEnv@0..48
            ItemBegin@0..20
              ClauseCommandName@0..6 "\\begin"
              ItemCurly@6..13
                TokenLBrace@6..7 "{"
                TokenWord@7..12 "array"
                TokenRBrace@12..13 "}"
              ClauseArgument@13..20
                ItemCurly@13..20
                  TokenLBrace@13..14 "{"
                  ItemText@14..16
                    TokenWord@14..16 "lc"
                  TokenRBrace@16..17 "}"
                  TokenLineBreak@17..18 "\n"
                  TokenWhiteSpace@18..20 "  "
            ItemText@20..22
              TokenWord@20..21 "a"
              TokenWhiteSpace@21..22 " "
            TokenAnd@22..23 "&"
            TokenWhiteSpace@23..24 " "
            ItemText@24..26
              TokenWord@24..25 "b"
              TokenWhiteSpace@25..26 " "
            ItemNewLine@26..28 "\\\\"
            TokenLineBreak@28..29 "\n"
            TokenWhiteSpace@29..31 "  "
            ItemText@31..33
              TokenWord@31..32 "c"
              TokenWhiteSpace@32..33 " "
            TokenAnd@33..34 "&"
            TokenWhiteSpace@34..35 " "
            ItemText@35..37
              TokenWord@35..36 "d"
              TokenLineBreak@36..37 "\n"
            ItemEnd@37..48
              ClauseCommandName@37..41 "\\end"
              ItemCurly@41..48
                TokenLBrace@41..42 "{"
                TokenWord@42..47 "array"
                TokenRBrace@47..48 "}"
        "###);
    }

    #[test]
    fn test_attachment() {
        // println!("{:#?}", parse(r#"{}_{1}^1"#));
        assert_debug_snapshot!(parse(r#"{}_{1}^2"#), @r###"
        ScopeRoot@0..8
          ItemAttachComponent@0..8
            ClauseArgument@0..6
              ItemAttachComponent@0..6
                ClauseArgument@0..2
                  ItemCurly@0..2
                    TokenLBrace@0..1 "{"
                    TokenRBrace@1..2 "}"
                TokenUnderline@2..3 "_"
                ItemCurly@3..6
                  TokenLBrace@3..4 "{"
                  ItemText@4..5
                    TokenWord@4..5 "1"
                  TokenRBrace@5..6 "}"
            TokenCaret@6..7 "^"
            TokenWord@7..8 "2"
        "###);
        assert_debug_snapshot!(parse(r#"\alpha_1"#), @r###"
        ScopeRoot@0..8
          ItemAttachComponent@0..8
            ClauseArgument@0..6
              ItemCmd@0..6
                ClauseCommandName@0..6 "\\alpha"
            TokenUnderline@6..7 "_"
            TokenWord@7..8 "1"
        "###);
        assert_debug_snapshot!(parse(r#"\alpha_[1]"#), @r###"
        ScopeRoot@0..10
          ItemAttachComponent@0..8
            ClauseArgument@0..6
              ItemCmd@0..6
                ClauseCommandName@0..6 "\\alpha"
            TokenUnderline@6..7 "_"
            TokenLBracket@7..8 "["
          ItemText@8..9
            TokenWord@8..9 "1"
          TokenRBracket@9..10 "]"
        "###);
        assert_debug_snapshot!(parse(r#"\alpha_(1)"#), @r###"
        ScopeRoot@0..10
          ItemAttachComponent@0..8
            ClauseArgument@0..6
              ItemCmd@0..6
                ClauseCommandName@0..6 "\\alpha"
            TokenUnderline@6..7 "_"
            TokenLParen@7..8 "("
          ItemText@8..9
            TokenWord@8..9 "1"
          TokenRParen@9..10 ")"
        "###);
        assert_debug_snapshot!(parse(r#"_1"#), @r###"
        ScopeRoot@0..2
          ItemAttachComponent@0..2
            TokenUnderline@0..1 "_"
            TokenWord@1..2 "1"
        "###);
        // Note: this is an invalid expression
        assert_debug_snapshot!(parse(r#"\over_1"#), @r###"
        ScopeRoot@0..7
          ItemCmd@0..7
            ClauseArgument@0..0
            ClauseCommandName@0..5 "\\over"
            ClauseArgument@5..7
              ItemAttachComponent@5..7
                TokenUnderline@5..6 "_"
                TokenWord@6..7 "1"
        "###);
        assert_debug_snapshot!(parse(r#"{}_1"#), @r###"
        ScopeRoot@0..4
          ItemAttachComponent@0..4
            ClauseArgument@0..2
              ItemCurly@0..2
                TokenLBrace@0..1 "{"
                TokenRBrace@1..2 "}"
            TokenUnderline@2..3 "_"
            TokenWord@3..4 "1"
        "###);
        assert_debug_snapshot!(parse(r#"{}_1_1"#), @r###"
        ScopeRoot@0..6
          ItemAttachComponent@0..6
            ClauseArgument@0..4
              ItemAttachComponent@0..4
                ClauseArgument@0..2
                  ItemCurly@0..2
                    TokenLBrace@0..1 "{"
                    TokenRBrace@1..2 "}"
                TokenUnderline@2..3 "_"
                TokenWord@3..4 "1"
            TokenUnderline@4..5 "_"
            TokenWord@5..6 "1"
        "###);
        assert_debug_snapshot!(parse(r#"\frac{1}{2}_{3}"#), @r###"
        ScopeRoot@0..15
          ItemAttachComponent@0..15
            ClauseArgument@0..11
              ItemCmd@0..11
                ClauseCommandName@0..5 "\\frac"
                ClauseArgument@5..8
                  ItemCurly@5..8
                    TokenLBrace@5..6 "{"
                    ItemText@6..7
                      TokenWord@6..7 "1"
                    TokenRBrace@7..8 "}"
                ClauseArgument@8..11
                  ItemCurly@8..11
                    TokenLBrace@8..9 "{"
                    ItemText@9..10
                      TokenWord@9..10 "2"
                    TokenRBrace@10..11 "}"
            TokenUnderline@11..12 "_"
            ItemCurly@12..15
              TokenLBrace@12..13 "{"
              ItemText@13..14
                TokenWord@13..14 "3"
              TokenRBrace@14..15 "}"
        "###);
        assert_debug_snapshot!(parse(r#"\overbrace{a + b + c}^{\text{This is an overbrace}}"#), @r###"
        ScopeRoot@0..51
          ItemCmd@0..10
            ClauseCommandName@0..10 "\\overbrace"
          ItemAttachComponent@10..51
            ClauseArgument@10..21
              ItemCurly@10..21
                TokenLBrace@10..11 "{"
                ItemText@11..20
                  TokenWord@11..12 "a"
                  TokenWhiteSpace@12..13 " "
                  TokenWord@13..14 "+"
                  TokenWhiteSpace@14..15 " "
                  TokenWord@15..16 "b"
                  TokenWhiteSpace@16..17 " "
                  TokenWord@17..18 "+"
                  TokenWhiteSpace@18..19 " "
                  TokenWord@19..20 "c"
                TokenRBrace@20..21 "}"
            TokenCaret@21..22 "^"
            ItemCurly@22..51
              TokenLBrace@22..23 "{"
              ItemCmd@23..28
                ClauseCommandName@23..28 "\\text"
              ItemCurly@28..50
                TokenLBrace@28..29 "{"
                ItemText@29..49
                  TokenWord@29..33 "This"
                  TokenWhiteSpace@33..34 " "
                  TokenWord@34..36 "is"
                  TokenWhiteSpace@36..37 " "
                  TokenWord@37..39 "an"
                  TokenWhiteSpace@39..40 " "
                  TokenWord@40..49 "overbrace"
                TokenRBrace@49..50 "}"
              TokenRBrace@50..51 "}"
        "###);
        assert_debug_snapshot!(parse(r#"\underbrace{x \times y}_{\text{This is an underbrace}}"#), @r###"
        ScopeRoot@0..54
          ItemCmd@0..11
            ClauseCommandName@0..11 "\\underbrace"
          ItemAttachComponent@11..54
            ClauseArgument@11..23
              ItemCurly@11..23
                TokenLBrace@11..12 "{"
                ItemText@12..14
                  TokenWord@12..13 "x"
                  TokenWhiteSpace@13..14 " "
                ItemCmd@14..20
                  ClauseCommandName@14..20 "\\times"
                TokenWhiteSpace@20..21 " "
                ItemText@21..22
                  TokenWord@21..22 "y"
                TokenRBrace@22..23 "}"
            TokenUnderline@23..24 "_"
            ItemCurly@24..54
              TokenLBrace@24..25 "{"
              ItemCmd@25..30
                ClauseCommandName@25..30 "\\text"
              ItemCurly@30..53
                TokenLBrace@30..31 "{"
                ItemText@31..52
                  TokenWord@31..35 "This"
                  TokenWhiteSpace@35..36 " "
                  TokenWord@36..38 "is"
                  TokenWhiteSpace@38..39 " "
                  TokenWord@39..41 "an"
                  TokenWhiteSpace@41..42 " "
                  TokenWord@42..52 "underbrace"
                TokenRBrace@52..53 "}"
              TokenRBrace@53..54 "}"
        "###);
        assert_debug_snapshot!(parse(r#"x_1''^2"#), @r###"
        ScopeRoot@0..7
          ItemAttachComponent@0..7
            ClauseArgument@0..5
              ItemAttachComponent@0..5
                ClauseArgument@0..4
                  ItemAttachComponent@0..4
                    ClauseArgument@0..3
                      ItemAttachComponent@0..3
                        ClauseArgument@0..1
                          ItemText@0..1
                            TokenWord@0..1 "x"
                        TokenUnderline@1..2 "_"
                        TokenWord@2..3 "1"
                    TokenApostrophe@3..4 "'"
                TokenApostrophe@4..5 "'"
            TokenCaret@5..6 "^"
            TokenWord@6..7 "2"
        "###);
        assert_debug_snapshot!(parse(r#"x''_1"#), @r###"
        ScopeRoot@0..5
          ItemAttachComponent@0..5
            ClauseArgument@0..3
              ItemAttachComponent@0..3
                ClauseArgument@0..2
                  ItemAttachComponent@0..2
                    ClauseArgument@0..1
                      ItemText@0..1
                        TokenWord@0..1 "x"
                    TokenApostrophe@1..2 "'"
                TokenApostrophe@2..3 "'"
            TokenUnderline@3..4 "_"
            TokenWord@4..5 "1"
        "###);
        assert_debug_snapshot!(parse(r#"''"#), @r###"
        ScopeRoot@0..2
          TokenApostrophe@0..1 "'"
          TokenApostrophe@1..2 "'"
        "###);
        assert_debug_snapshot!(parse(r#"\frac''"#), @r###"
        ScopeRoot@0..7
          ItemCmd@0..7
            ClauseCommandName@0..5 "\\frac"
            ClauseArgument@5..6
              TokenApostrophe@5..6 "'"
            ClauseArgument@6..7
              TokenApostrophe@6..7 "'"
        "###);
    }

    #[test]
    fn test_attachment_may_weird() {
        assert_debug_snapshot!(parse(r#"\frac ab_c"#), @r###"
        ScopeRoot@0..10
          ItemAttachComponent@0..10
            ClauseArgument@0..8
              ItemCmd@0..8
                ClauseCommandName@0..5 "\\frac"
                TokenWhiteSpace@5..6 " "
                ClauseArgument@6..7
                  TokenWord@6..7 "a"
                ClauseArgument@7..8
                  TokenWord@7..8 "b"
            TokenUnderline@8..9 "_"
            TokenWord@9..10 "c"
        "###);
        assert_debug_snapshot!(parse(r#"\frac a_c b"#), @r###"
        ScopeRoot@0..11
          ItemAttachComponent@0..9
            ClauseArgument@0..7
              ItemCmd@0..7
                ClauseCommandName@0..5 "\\frac"
                TokenWhiteSpace@5..6 " "
                ClauseArgument@6..7
                  TokenWord@6..7 "a"
            TokenUnderline@7..8 "_"
            TokenWord@8..9 "c"
          TokenWhiteSpace@9..10 " "
          ItemText@10..11
            TokenWord@10..11 "b"
        "###);
        assert_debug_snapshot!(parse(r#"\frac {a_c} b"#), @r###"
        ScopeRoot@0..13
          ItemCmd@0..13
            ClauseCommandName@0..5 "\\frac"
            TokenWhiteSpace@5..6 " "
            ClauseArgument@6..12
              ItemCurly@6..12
                TokenLBrace@6..7 "{"
                ItemAttachComponent@7..10
                  ClauseArgument@7..8
                    ItemText@7..8
                      TokenWord@7..8 "a"
                  TokenUnderline@8..9 "_"
                  TokenWord@9..10 "c"
                TokenRBrace@10..11 "}"
                TokenWhiteSpace@11..12 " "
            ClauseArgument@12..13
              TokenWord@12..13 "b"
        "###);
    }

    #[test]
    fn test_sqrt() {
        assert_debug_snapshot!(parse(r#"\sqrt 12"#), @r###"
        ScopeRoot@0..8
          ItemCmd@0..7
            ClauseCommandName@0..5 "\\sqrt"
            TokenWhiteSpace@5..6 " "
            ClauseArgument@6..7
              TokenWord@6..7 "1"
          ItemText@7..8
            TokenWord@7..8 "2"
        "###);
        assert_debug_snapshot!(parse(r#"\sqrt{1}2"#), @r###"
        ScopeRoot@0..9
          ItemCmd@0..8
            ClauseCommandName@0..5 "\\sqrt"
            ClauseArgument@5..8
              ItemCurly@5..8
                TokenLBrace@5..6 "{"
                ItemText@6..7
                  TokenWord@6..7 "1"
                TokenRBrace@7..8 "}"
          ItemText@8..9
            TokenWord@8..9 "2"
        "###);
        // Note: this is an invalid expression
        assert_debug_snapshot!(parse(r#"\sqrt[1]"#), @r###"
        ScopeRoot@0..8
          ItemCmd@0..8
            ClauseCommandName@0..5 "\\sqrt"
            ClauseArgument@5..8
              ItemBracket@5..8
                TokenLBracket@5..6 "["
                ItemText@6..7
                  TokenWord@6..7 "1"
                TokenRBracket@7..8 "]"
        "###);
        assert_debug_snapshot!(parse(r#"\sqrt[1]{2}"#), @r###"
        ScopeRoot@0..11
          ItemCmd@0..11
            ClauseCommandName@0..5 "\\sqrt"
            ClauseArgument@5..8
              ItemBracket@5..8
                TokenLBracket@5..6 "["
                ItemText@6..7
                  TokenWord@6..7 "1"
                TokenRBracket@7..8 "]"
            ClauseArgument@8..11
              ItemCurly@8..11
                TokenLBrace@8..9 "{"
                ItemText@9..10
                  TokenWord@9..10 "2"
                TokenRBrace@10..11 "}"
        "###);
        assert_debug_snapshot!(parse(r#"\sqrt[1]{2}3"#), @r###"
        ScopeRoot@0..12
          ItemCmd@0..11
            ClauseCommandName@0..5 "\\sqrt"
            ClauseArgument@5..8
              ItemBracket@5..8
                TokenLBracket@5..6 "["
                ItemText@6..7
                  TokenWord@6..7 "1"
                TokenRBracket@7..8 "]"
            ClauseArgument@8..11
              ItemCurly@8..11
                TokenLBrace@8..9 "{"
                ItemText@9..10
                  TokenWord@9..10 "2"
                TokenRBrace@10..11 "}"
          ItemText@11..12
            TokenWord@11..12 "3"
        "###);
    }
}
