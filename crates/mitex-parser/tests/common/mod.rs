pub mod parser {
    use mitex_parser::syntax::SyntaxNode;
    use mitex_spec_gen::DEFAULT_SPEC;

    use super::SnapNode;

    pub fn parse(input: &str) -> SyntaxNode {
        mitex_parser::parse(input, DEFAULT_SPEC.clone())
    }

    pub fn parse_snap(input: &str) -> SnapNode {
        super::ast_snapshot::SnapNode(parse(input))
    }
}

pub use ast_snapshot::{SnapNode, SnapToken};
pub use parser::*;

pub mod ast_snapshot {
    use core::fmt;
    use std::fmt::Write;

    use mitex_parser::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};
    use rowan::NodeOrToken;

    pub struct SnapNode(pub SyntaxNode);

    impl fmt::Debug for SnapNode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut p = AstPrinter { level: 0 };
            p.show_node(self.0.clone(), f)
        }
    }

    pub struct SnapToken(pub SyntaxToken);

    impl fmt::Debug for SnapToken {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut p = AstPrinter { level: 0 };
            p.show_token(self.0.clone(), f)
        }
    }

    impl fmt::Display for SnapToken {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut p = AstPrinter { level: 0 };
            p.show_token(self.0.clone(), f)
        }
    }

    struct AstPrinter {
        level: usize,
    }

    impl AstPrinter {
        fn pretty_syntax_kind(f: &mut fmt::Formatter<'_>, sk: SyntaxKind) -> fmt::Result {
            let w = match sk {
                SyntaxKind::TokenError => "error'",
                SyntaxKind::TokenLineBreak => "br'",
                SyntaxKind::TokenWhiteSpace => "space'",
                SyntaxKind::TokenComment => "comment'",
                SyntaxKind::TokenLBrace => "lbrace'",
                SyntaxKind::TokenRBrace => "rbrace'",
                SyntaxKind::TokenLBracket => "lbracket'",
                SyntaxKind::TokenRBracket => "rbracket'",
                SyntaxKind::TokenLParen => "lparen'",
                SyntaxKind::TokenRParen => "rparen'",
                SyntaxKind::TokenComma => "comma'",
                SyntaxKind::TokenTilde => "tilde'",
                SyntaxKind::TokenSlash => "slash'",
                SyntaxKind::TokenWord => "word'",
                SyntaxKind::TokenDollar => "dollar'",
                SyntaxKind::TokenBeginMath => "begin-math'",
                SyntaxKind::TokenEndMath => "end-math",
                SyntaxKind::TokenAmpersand => "ampersand'",
                SyntaxKind::TokenHash => "hash'",
                SyntaxKind::TokenAsterisk => "asterisk'",
                SyntaxKind::TokenAtSign => "at-sign'",
                SyntaxKind::TokenUnderscore => "underscore'",
                SyntaxKind::TokenCaret => "caret'",
                SyntaxKind::TokenApostrophe => "apostrophe'",
                SyntaxKind::TokenDitto => "ditto'",
                SyntaxKind::TokenSemicolon => "semicolon'",
                SyntaxKind::TokenCommandSym => "sym'",
                SyntaxKind::ClauseCommandName => "cmd-name",
                SyntaxKind::ClauseArgument => "args",
                SyntaxKind::ClauseLR => "clause-lr",
                SyntaxKind::ItemNewLine => "newline",
                SyntaxKind::ItemText => "text",
                SyntaxKind::ItemCurly => "curly",
                SyntaxKind::ItemBracket => "bracket",
                SyntaxKind::ItemParen => "paren",
                SyntaxKind::ItemCmd => "cmd",
                SyntaxKind::ItemEnv => "env",
                SyntaxKind::ItemLR => "lr",
                SyntaxKind::ItemBegin => "begin",
                SyntaxKind::ItemEnd => "end",
                SyntaxKind::ItemBlockComment => "block-comment",
                SyntaxKind::ItemTypstCode => "embedded-code",
                SyntaxKind::ItemAttachComponent => "attach-comp",
                SyntaxKind::ItemFormula => "formula",
                SyntaxKind::ScopeRoot => "root",
            };

            f.write_str(w)
        }

        fn show_node(&mut self, node: SyntaxNode, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Self::pretty_syntax_kind(f, node.kind())?;

            if f.alternate() {
                if node.children().count() == 0 {
                    let mut first = true;
                    f.write_char('(')?;
                    for tok in node.children_with_tokens() {
                        if first {
                            first = false;
                        } else {
                            f.write_char(',')?
                        }
                        match tok {
                            NodeOrToken::Node(_) => unreachable!(),
                            NodeOrToken::Token(token) => self.show_token(token, f)?,
                        }
                    }

                    f.write_char(')')?;
                    f.write_char('\n')?;
                    return Ok(());
                }

                f.write_char('\n')?;
                // Print with children
                self.level += 1;
                for element in node.children_with_tokens() {
                    for _ in 0..self.level {
                        f.write_char('|')?;
                    }
                    match element {
                        NodeOrToken::Node(sub) => self.show_node(sub, f)?,
                        NodeOrToken::Token(token) => {
                            self.show_token(token, f)?;
                            f.write_char('\n')?
                        }
                    }
                }
                self.level -= 1;
            }
            Ok(())
        }

        fn show_token(&mut self, token: SyntaxToken, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Self::pretty_syntax_kind(f, token.kind())?;

            if token.text().len() < 25 {
                return write!(f, "({:?})", token.text());
            }
            let text = token.text();
            for idx in 21..25 {
                if text.is_char_boundary(idx) {
                    return write!(f, "({:?}..)", &text[..idx]);
                }
            }
            unreachable!()
        }
    }
}
