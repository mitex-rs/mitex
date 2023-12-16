use itertools::{EitherOrBoth, Itertools};
use rowan::ast::AstNode;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    ERROR = 0,

    LINE_BREAK,
    WHITESPACE,
    COMMENT,
    VERBATIM,
    L_CURLY,
    R_CURLY,
    L_BRACK,
    R_BRACK,
    L_PAREN,
    R_PAREN,
    COMMA,
    EQUALITY_SIGN,
    WORD,
    DOLLAR,
    COMMAND_NAME,

    PREAMBLE,
    TEXT,
    KEY,
    CURLY_GROUP,
    CURLY_GROUP_WORD,
    CURLY_GROUP_COMMAND,
    BRACK_GROUP,
    MIXED_GROUP,
    GENERIC_COMMAND,
    ENVIRONMENT,
    BEGIN,
    END,
    EQUATION,
    FORMULA,
    MATH_OPERATOR,
    COLOR_REFERENCE,
    BLOCK_COMMENT,
    ROOT,
}

use SyntaxKind::*;

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LatexLanguage {}

impl rowan::Language for LatexLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<LatexLanguage>;

pub type SyntaxToken = rowan::SyntaxToken<LatexLanguage>;

pub type SyntaxElement = rowan::SyntaxElement<LatexLanguage>;

macro_rules! cst_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Language = LatexLanguage;

            fn can_cast(kind: SyntaxKind) -> bool {
                match kind {
                    $($kind => true,)+
                    _ => false,
                }
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

cst_node!(GenericCommand, GENERIC_COMMAND);

impl GenericCommand {
    pub fn name(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == COMMAND_NAME)
    }
}

cst_node!(Environment, ENVIRONMENT);

impl Environment {
    pub fn begin(&self) -> Option<Begin> {
        self.syntax().children().find_map(Begin::cast)
    }

    pub fn end(&self) -> Option<End> {
        self.syntax().children().find_map(End::cast)
    }
}

cst_node!(Begin, BEGIN);

impl Begin {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }

    pub fn options(&self) -> Option<BrackGroup> {
        self.syntax().children().find_map(BrackGroup::cast)
    }
}

cst_node!(End, END);

impl End {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<CurlyGroupWord> {
        self.syntax().children().find_map(CurlyGroupWord::cast)
    }
}

cst_node!(CurlyGroupWord, CURLY_GROUP_WORD);

// impl HasCurly for CurlyGroupWord {}

impl CurlyGroupWord {
    pub fn key(&self) -> Option<Key> {
        self.syntax().children().find_map(Key::cast)
    }
}

cst_node!(BrackGroup, BRACK_GROUP);

impl BrackGroup {}

cst_node!(Key, KEY);

impl Key {
    pub fn words(&self) -> impl Iterator<Item = SyntaxToken> {
        use SyntaxKind::*;
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| !matches!(node.kind(), WHITESPACE | LINE_BREAK | COMMENT))
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.words()
            .zip_longest(other.words())
            .all(|result| match result {
                EitherOrBoth::Both(left, right) => left.text() == right.text(),
                EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => false,
            })
    }
}

impl Eq for Key {}

impl ToString for Key {
    fn to_string(&self) -> String {
        use SyntaxKind::*;
        let mut buf = String::new();
        for token in self
            .syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
        {
            if matches!(token.kind(), WHITESPACE | LINE_BREAK | COMMENT) {
                buf.push(' ');
            } else {
                buf.push_str(token.text());
            }
        }

        buf = String::from(buf.trim());
        buf
    }
}

// impl HasBrack for BrackGroup {}
