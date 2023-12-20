mod arg_match;
pub mod parser;
pub use mitex_spec as spec;
pub mod syntax;

pub use parser::parse;
pub use spec::*;

pub mod command_preludes {
    use crate::{ArgShape, CommandSpecItem};

    pub fn define_command(num: u8) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(num)),
            alias: None,
        })
    }

    pub fn define_glob_command(reg: &str, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::Glob(reg.into())),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_symbol(alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::None),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_command_with_alias(num: u8, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(num)),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_greedy_command(alias: &str) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape {
            args: crate::ArgShape::Right(crate::ArgPattern::Greedy),
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_matrix_env(num: Option<u8>, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Env(crate::EnvShape {
            args: num
                .map(crate::ArgPattern::FixedLenTerm)
                .unwrap_or(crate::ArgPattern::None),
            ctx_feature: crate::ContextFeature::IsMatrix,
            alias: Some(alias.to_owned()),
        })
    }

    pub fn define_normal_env(num: Option<u8>, alias: &str) -> CommandSpecItem {
        CommandSpecItem::Env(crate::EnvShape {
            args: num
                .map(crate::ArgPattern::FixedLenTerm)
                .unwrap_or(crate::ArgPattern::None),
            ctx_feature: crate::ContextFeature::None,
            alias: Some(alias.to_owned()),
        })
    }
    pub const fn define_const_command(args: ArgShape) -> CommandSpecItem {
        CommandSpecItem::Cmd(crate::CmdShape { args, alias: None })
    }

    pub const TEX_CMD0: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(0)));
    pub const TEX_CMD1: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(1)));
    pub const TEX_CMD2: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::FixedLenTerm(2)));
    pub const TEX_SYMBOL: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::None));
    pub const TEX_LEFT1_OPEARTOR: CommandSpecItem = define_const_command(crate::ArgShape::Left1);
    pub const TEX_GREEDY_OPERATOR: CommandSpecItem =
        define_const_command(crate::ArgShape::Right(crate::ArgPattern::Greedy));
    pub const TEX_INFIX_OPERATOR: CommandSpecItem =
        define_const_command(crate::ArgShape::InfixGreedy);
    pub const TEX_MATRIX_ENV: CommandSpecItem = CommandSpecItem::Env(crate::EnvShape {
        args: crate::ArgPattern::None,
        ctx_feature: crate::ContextFeature::IsMatrix,
        alias: None,
    });
    pub const TEX_NORMAL_ENV: CommandSpecItem = CommandSpecItem::Env(crate::EnvShape {
        args: crate::ArgPattern::None,
        ctx_feature: crate::ContextFeature::None,
        alias: None,
    });

    #[derive(Default)]
    pub struct SpecBuilder {
        commands: std::collections::HashMap<String, CommandSpecItem>,
    }

    impl SpecBuilder {
        pub fn add_command(&mut self, name: &str, item: CommandSpecItem) -> &mut Self {
            self.commands.insert(name.to_owned(), item);
            self
        }

        pub fn build(self) -> crate::CommandSpec {
            crate::CommandSpec::new(self.commands)
        }
    }
}

#[cfg(test)]
mod tests {

    use insta::assert_debug_snapshot;
    use rowan::ast::AstNode;

    use crate::{
        syntax::{CmdItem, EnvItem, LRItem, SyntaxKind, SyntaxNode},
        CommandSpec,
    };

    pub fn parse(input: &str) -> SyntaxNode {
        super::parse(input, DEFAULT_SPEC.clone())
    }

    // The default spec used for testing
    fn default_spec() -> CommandSpec {
        use super::command_preludes::*;
        use super::*;
        let mut builder = SpecBuilder::default();
        builder.add_command("underline", define_command(1));
        builder.add_command("mathrm", define_command_with_alias(1, "upright"));
        builder.add_command("frac", define_command(2));
        builder.add_command("alpha", TEX_SYMBOL);
        builder.add_command("sum", TEX_SYMBOL);
        builder.add_command("limits", TEX_LEFT1_OPEARTOR);
        builder.add_command("displaystyle", define_greedy_command("display"));
        builder.add_command(
            "over",
            CommandSpecItem::Cmd(CmdShape {
                args: ArgShape::InfixGreedy,
                alias: Some("frac".to_owned()),
            }),
        );
        builder.add_command("matrix", define_matrix_env(None, "matrix"));
        builder.add_command("pmatrix", define_matrix_env(None, "pmatrix"));
        builder.add_command("array", define_matrix_env(Some(1), "mitexarray"));

        builder.add_command("sqrt", define_glob_command("{,b}t", "mitexsqrt"));
        builder.build()
    }

    static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> =
        once_cell::sync::Lazy::new(default_spec);

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
    fn test_cmd_infix_bug() {
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
    }

    #[test]
    fn test_cmd_infix() {
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
