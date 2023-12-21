pub mod parser {
    use mitex_parser::syntax::{LatexLanguage, SyntaxNode};
    use mitex_spec::CommandSpec;

    pub type AstSnapshot = super::ast_snapshot::AstSnapshot<LatexLanguage>;

    pub fn parse(input: &str) -> SyntaxNode {
        mitex_parser::parse(input, DEFAULT_SPEC.clone())
    }

    pub fn parse_snap(input: &str) -> AstSnapshot {
        super::ast_snapshot::AstSnapshot(parse(input))
    }

    static DEFAULT_SPEC: once_cell::sync::Lazy<CommandSpec> = once_cell::sync::Lazy::new(|| {
        CommandSpec::from_bytes(include_bytes!(
            "../../../../target/mitex-artifacts/spec/default.rkyv"
        ))
    });
}

pub use parser::*;

pub mod ast_snapshot {
    use core::fmt;

    use rowan::{Language, NodeOrToken, SyntaxNode, SyntaxToken, WalkEvent};

    pub struct AstSnapshot<L: Language>(pub SyntaxNode<L>);

    impl<L: Language> AstSnapshot<L> {
        fn show_node(node: SyntaxNode<L>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if f.alternate() {
                let mut level = 0;
                for event in node.preorder_with_tokens() {
                    match event {
                        WalkEvent::Enter(element) => {
                            for _ in 0..level {
                                write!(f, " ")?;
                            }
                            match element {
                                NodeOrToken::Node(sub) => Self::show_node(sub, f)?,
                                NodeOrToken::Token(token) => Self::show_token(token, f)?,
                            }
                            level += 1;
                        }
                        WalkEvent::Leave(_) => level -= 1,
                    }
                }
                assert_eq!(level, 0);
                Ok(())
            } else {
                write!(f, "{:?}", node.kind())
            }
        }

        fn show_token(token: SyntaxToken<L>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", token.kind())?;
            if token.text().len() < 25 {
                return write!(f, " {:?}", token.text());
            }
            let text = token.text();
            for idx in 21..25 {
                if text.is_char_boundary(idx) {
                    let text = format!("{} ...", &text[..idx]);
                    return write!(f, " {:?}", text);
                }
            }
            unreachable!()
        }
    }

    impl<L: Language> fmt::Debug for AstSnapshot<L> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            AstSnapshot::show_node(self.0.clone(), f)
        }
    }
}
