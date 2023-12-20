use mitex_lexer::{BraceKind, Token};
use rowan::ast::AstNode;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens
    TokenError = 0,
    TokenLineBreak,
    TokenWhiteSpace,
    TokenComment,
    TokenLBrace,
    TokenRBrace,
    TokenLBracket,
    TokenRBracket,
    TokenLParen,
    TokenRParen,
    TokenComma,
    TokenTilde,
    TokenDivide,
    TokenEqual,
    TokenWord,
    TokenDollar,
    TokenAnd,
    TokenUnderline,
    TokenCaret,
    TokenApostrophe,
    TokenCommandSym,

    // Clauses
    ClauseCommandName,
    ClauseArgKey,
    ClauseArgument,
    ClauseLR,

    // Items
    ItemNewLine,
    ItemText,
    ItemCurly,
    ItemBracket,
    ItemParen,
    ItemCmd,
    ItemEnv,
    ItemLR,
    ItemBegin,
    ItemEnd,
    ItemBlockComment,
    ItemAttachComponent,
    ItemFormula,

    // Scopes
    ScopeRoot,
}

impl From<Token> for SyntaxKind {
    fn from(kind: Token) -> Self {
        match kind {
            Token::LineBreak => SyntaxKind::TokenLineBreak,
            Token::Whitespace => SyntaxKind::TokenWhiteSpace,
            Token::LineComment => SyntaxKind::TokenComment,
            Token::Left(BraceKind::Curly) => SyntaxKind::TokenLBrace,
            Token::Right(BraceKind::Curly) => SyntaxKind::TokenRBrace,
            Token::Left(BraceKind::Bracket) => SyntaxKind::TokenLBracket,
            Token::Right(BraceKind::Bracket) => SyntaxKind::TokenRBracket,
            Token::Left(BraceKind::Paren) => SyntaxKind::TokenLParen,
            Token::Right(BraceKind::Paren) => SyntaxKind::TokenRParen,
            Token::Comma => SyntaxKind::TokenComma,
            Token::Tilde => SyntaxKind::TokenTilde,
            Token::Divide => SyntaxKind::TokenDivide,
            Token::Equal => SyntaxKind::TokenEqual,
            Token::Underline => SyntaxKind::TokenUnderline,
            Token::Apostrophe => SyntaxKind::TokenApostrophe,
            Token::Caret => SyntaxKind::TokenCaret,
            Token::Word => SyntaxKind::TokenWord,
            Token::Dollar => SyntaxKind::TokenDollar,
            Token::And => SyntaxKind::TokenAnd,
            Token::NewLine => SyntaxKind::ItemNewLine,
            Token::CommandName(_) => SyntaxKind::ClauseCommandName,
        }
    }
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            SyntaxKind::TokenLineBreak
                | SyntaxKind::TokenWhiteSpace
                | SyntaxKind::TokenComment
                | SyntaxKind::ItemNewLine
        )
    }
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
        assert!(raw.0 <= ScopeRoot as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<LatexLanguage>;

pub type SyntaxToken = rowan::SyntaxToken<LatexLanguage>;

pub type SyntaxElement = rowan::SyntaxElement<LatexLanguage>;

macro_rules! syntax_tree_node {
    ($(#[$attr:meta])* $name:ident, $($kind:pat),+) => {
        #[derive(Clone)]
        #[repr(transparent)]
        $(#[$attr])*
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

syntax_tree_node!(
    /// Command item in latex document
    ///
    /// In short it is in shape of
    /// ```coffeescript
    /// ItemCmd(
    ///   ClauseArgument(rev-arg1)?
    ///   ClauseCommandName(name),
    ///   ClauseArgument(arg1), ...
    /// )
    /// ```
    ///
    /// Exmaple:
    /// ```latex
    /// \documentclass{article}
    /// ```
    ///
    /// Which will be parsed as:
    /// ```coffeescript
    /// ItemCmd(
    ///   ClauseCommandName(
    ///    TokenWord("documentclass")
    ///  ),
    ///   ClauseArgument(
    ///     ItemCurly(
    ///      TokenLBrace,
    ///     ItemText(
    ///      TokenWord("article")
    ///    ),
    ///     TokenRBrace
    ///   )
    /// )
    /// ```
    CmdItem,
    ItemCmd
);

impl CmdItem {
    /// Get the token corresponding to command name
    pub fn name_tok(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == ClauseCommandName)
    }

    /// Get the command arguments
    pub fn arguments(&self) -> impl Iterator<Item = SyntaxNode> {
        self.syntax()
            .children()
            .filter(|node| node.kind() == ClauseArgument)
    }
}

syntax_tree_node!(
    /// Environment item in latex document
    /// ```coffeescript
    /// ItemBegin(
    ///   ClauseCommandName(name),
    ///   ClauseArgument(arg1), ...
    /// )
    /// ...
    /// ItemEnd(
    ///   ClauseCommandName(name),
    /// )
    EnvItem,
    ItemEnv
);

impl EnvItem {
    /// Get the begin clause of the environment
    pub fn begin(&self) -> Option<BeginItem> {
        self.syntax().children().find_map(BeginItem::cast)
    }

    /// Get the end clause of the environment
    pub fn end(&self) -> Option<EndItem> {
        self.syntax().children().find_map(EndItem::cast)
    }

    /// Get the name of the environment
    pub fn name_tok(&self) -> Option<SyntaxToken> {
        self.begin()
            .and_then(|begin| begin.name())
            .and_then(|name| name.key())
    }

    /// Get the arguments of the environment
    pub fn arguments(&self) -> impl Iterator<Item = SyntaxNode> {
        self.begin().into_iter().flat_map(|begin| begin.arguments())
    }

    /// Get the options of the environment
    pub fn options(&self) -> Option<BracketItem> {
        self.begin().and_then(|begin| begin.options())
    }
}

syntax_tree_node!(LRItem, ItemLR);

impl LRItem {
    /// Get the left clause
    pub fn left(&self) -> Option<LRClause> {
        self.syntax().first_child().and_then(LRClause::cast)
    }
    /// Get the right clause
    pub fn right(&self) -> Option<LRClause> {
        self.syntax().last_child().and_then(LRClause::cast)
    }

    /// Get the left symbol wrapped in the clause
    pub fn left_sym(&self) -> Option<SyntaxToken> {
        self.left().and_then(|clause| clause.sym())
    }

    /// Get the right symbol wrapped in the clause
    pub fn right_sym(&self) -> Option<SyntaxToken> {
        self.right().and_then(|clause| clause.sym())
    }
}

syntax_tree_node!(LRClause, ClauseLR);

impl LRClause {
    /// Get the command kind
    pub fn is_left(&self) -> bool {
        self.syntax()
            .first_token()
            .map(|node| node.kind() == ClauseCommandName && node.text() == "\\left")
            .unwrap_or(false)
    }

    /// Get the symbol wrapped in the clause
    pub fn sym(&self) -> Option<SyntaxToken> {
        self.syntax()
            .last_token()
            .filter(|node| !matches!(node.kind(), ClauseCommandName))
    }
}

syntax_tree_node!(BeginItem, ItemBegin);

impl BeginItem {
    /// Get the command in the begin clause
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    /// Get the name in the begin clause
    pub fn name(&self) -> Option<CurlyWordItem> {
        self.syntax().children().find_map(CurlyWordItem::cast)
    }

    /// Get the options of the environment
    pub fn options(&self) -> Option<BracketItem> {
        todo!()
    }

    /// Get the arguments of the environment
    pub fn arguments(&self) -> impl Iterator<Item = SyntaxNode> {
        self.syntax()
            .children()
            .filter(|node| node.kind() == ClauseArgument)
    }
}

syntax_tree_node!(EndItem, ItemEnd);

impl EndItem {
    /// Get the command in the end clause
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    /// Get the name in the end clause
    pub fn name(&self) -> Option<CurlyWordItem> {
        self.syntax().children().find_map(CurlyWordItem::cast)
    }
}

syntax_tree_node!(CurlyWordItem, ItemCurly);

impl CurlyWordItem {
    /// Get the word in the curly item
    pub fn key(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == TokenWord)
    }
}

syntax_tree_node!(BracketItem, ItemBracket);

impl BracketItem {}
