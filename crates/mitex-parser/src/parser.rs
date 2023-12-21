use rowan::{Checkpoint, GreenNode, GreenNodeBuilder};

use crate::arg_match::{ArgMatcher, ArgMatcherBuilder};
use crate::spec::argument_kind::*;
use crate::syntax::{
    SyntaxKind::{self, *},
    SyntaxNode,
};
use crate::{ArgPattern, ArgShape, CommandSpec};
use mitex_lexer::{BraceKind, CommandName, Lexer, Token};

/// Stacked scope for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseScope {
    /// The root scope, this is set when the parser enters the entry point
    Root,
    /// The scope of a formula, i.e. `$...$` or `$$...$$`
    Formula,
    /// The scope of an environment, i.e. `\begin{...}...\end{...}`
    Environment,
    /// The scope of a lr command, i.e. `\left...\right`
    LR,
    /// The scope of a curly group, i.e. `{...}`
    CurlyItem,
    /// The scope of a bracket group, i.e. `[...]`
    BracketItem,
    /// The scope of a parenthesis group, i.e. `(...)`
    ParenItem,
}

mod list_state {
    use std::mem::MaybeUninit;

    use rowan::Checkpoint;

    use super::ParseScope;

    /// State used by list parsers
    #[derive(Debug, Clone, Copy)]
    pub struct ListState {
        /// The checkpoint of the first item in the list
        /// Note: if an infix command is parsed, this will become None
        start: MaybeUninit<Checkpoint>,
        /// The checkpoint of the last item in the list
        last: MaybeUninit<Checkpoint>,
        /// The current scope
        pub scope: ParseScope,
        /// Whether a start position of the list is valid
        has_start: bool,
        /// Whether a last position of the list is valid
        has_last: bool,
    }

    impl Default for ListState {
        fn default() -> Self {
            Self {
                start: MaybeUninit::uninit(),
                last: MaybeUninit::uninit(),
                scope: ParseScope::Root,
                has_start: false,
                has_last: false,
            }
        }
    }

    impl ListState {
        /// Create a new list state with the given scope
        pub fn new(scope: ParseScope) -> Self {
            Self {
                scope,
                ..Default::default()
            }
        }

        /// The start position of the list
        #[inline]
        pub fn start(&self) -> Option<Checkpoint> {
            self.has_start.then(|| unsafe { self.start.assume_init() })
        }

        /// The last position of the list
        #[inline]
        pub fn last(&self) -> Option<Checkpoint> {
            self.has_last.then(|| unsafe { self.last.assume_init() })
        }

        /// Take the start position of the list
        /// This will set the `has_start` flag to false
        #[inline]
        pub fn take_start(&mut self) -> Option<Checkpoint> {
            let start = self.start();
            self.has_start = false;
            start
        }

        /// Store the start position of the list
        pub fn store_start(&mut self, current: Checkpoint) {
            self.has_start = true;
            self.start = MaybeUninit::new(current);
        }

        /// Store the last position of the list
        pub fn store_last(&mut self, current: Checkpoint) {
            self.has_last = true;
            self.last = MaybeUninit::new(current);
        }

        /// Store the last position of the list
        pub fn may_store_start(&mut self, current: Option<Checkpoint>) {
            if let Some(current) = current {
                self.store_start(current);
            } else {
                self.has_start = false;
            }
        }

        /// Store the last position of the list
        pub fn may_store_last(&mut self, current: Option<Checkpoint>) {
            if let Some(current) = current {
                self.store_last(current);
            } else {
                self.has_last = false;
            }
        }
    }
}
use list_state::ListState;

/// The mutable parser that parse the input text into a syntax tree
#[derive(Debug)]
struct Parser<'a> {
    /// Lexer level structure
    lexer: Lexer<'a>,
    /// Helper for building syntax tree
    builder: GreenNodeBuilder<'static>,

    /// Command specification
    spec: CommandSpec,
    /// Argument matcher builder containing cached regexes
    arg_matchers: ArgMatcherBuilder,

    /// State used by item_list/argument_list parser
    /// The current state
    list_state: ListState,
}

impl<'a> Parser<'a> {
    /// Create a new parser borrowing the input text and the immutable command
    /// specification.
    pub fn new(text: &'a str, spec: CommandSpec) -> Self {
        Self {
            lexer: Lexer::new(text, spec.clone()),
            builder: GreenNodeBuilder::new(),
            spec,
            arg_matchers: ArgMatcherBuilder::default(),
            list_state: Default::default(),
        }
    }

    /// List State
    /// The start position of the list
    #[inline]
    fn list_start(&self) -> Option<Checkpoint> {
        self.list_state.start()
    }

    /// List State
    /// The last position of the list
    #[inline]
    fn list_last(&self) -> Option<Checkpoint> {
        self.list_state.last()
    }

    /// List State
    /// The current scope
    #[inline]
    fn scope(&self) -> ParseScope {
        self.list_state.scope
    }

    /// Lexer Interface
    /// Peek the next token
    fn peek(&self) -> Option<Token> {
        self.lexer.peek()
    }

    /// Lexer Interface
    /// Consume the next token and attach it to the syntax tree
    fn eat(&mut self) {
        let (kind, text) = self.lexer.eat().unwrap();
        let kind: SyntaxKind = kind.into();
        self.builder.token(kind.into(), text);
    }

    /// Lexer Interface
    /// Consume the next token and attach it to the syntax tree with another
    /// syntax kind
    fn eat_as(&mut self, kind: SyntaxKind) {
        let (_, text) = self.lexer.eat().unwrap();
        self.builder.token(kind.into(), text);
    }

    /// Lexer Interface
    /// Consume the next token if it matches the given kind
    fn eat_if(&mut self, kind: Token) {
        if self.peek() == Some(kind) {
            self.eat();
            self.trivia();
        }
    }

    /// Lexer Interface
    fn single_char(&mut self) -> Option<()> {
        let first_char = self.lexer.peek_char()?;
        self.builder
            .token(TokenWord.into(), &first_char.to_string());
        self.lexer.consume_word(1);

        Some(())
    }

    /// Lexer Interface
    /// Consume tokens until the next non-trivia token
    fn trivia(&mut self) {
        fn is_trivia(kind: Token) -> bool {
            use Token::*;
            matches!(kind, LineBreak | Whitespace | LineComment)
        }

        while self.peek().map_or(false, is_trivia) {
            self.eat();
        }
    }

    /// Entry point
    /// The main entry point of the parser
    pub fn parse(mut self) -> GreenNode {
        self.builder.start_node(ScopeRoot.into());
        self.item_list(ParseScope::Root);
        self.builder.finish_node();
        self.builder.finish()
    }

    /// Parsing Helper
    /// Check if the parser should stop parsing the current item
    #[inline]
    fn stop_by_scope(&mut self, kind: Token) -> bool {
        match self.scope() {
            ParseScope::Root => false,
            ParseScope::Formula => matches!(
                kind,
                Token::Right(BraceKind::Curly)
                    | Token::CommandName(CommandName::EndEnvironment | CommandName::Right)
                    | Token::Dollar
            ),
            ParseScope::Environment => matches!(
                kind,
                Token::Right(BraceKind::Curly) | Token::CommandName(CommandName::EndEnvironment)
            ),
            ParseScope::CurlyItem => matches!(kind, Token::Right(BraceKind::Curly)),
            ParseScope::BracketItem => matches!(
                kind,
                Token::Right(BraceKind::Curly | BraceKind::Bracket)
                    | Token::CommandName(CommandName::EndEnvironment | CommandName::Right)
            ),
            ParseScope::ParenItem => matches!(
                kind,
                Token::Right(BraceKind::Curly | BraceKind::Paren)
                    | Token::CommandName(CommandName::EndEnvironment | CommandName::Right)
            ),
            ParseScope::LR => matches!(
                kind,
                Token::Right(BraceKind::Curly)
                    | Token::CommandName(CommandName::EndEnvironment | CommandName::Right)
            ),
        }
    }

    /// Parsing Helper
    /// Parse a list of items which also maintains the `list_start` and
    /// `list_last` state for inner item parsers
    #[inline]
    fn item_list(&mut self, scope: ParseScope) {
        let parent_state = self.list_state;

        let mut current = self.builder.checkpoint();
        self.list_state = ListState::new(scope);
        self.list_state.store_start(current);

        while self.peek().map_or(false, |kind| !self.stop_by_scope(kind)) {
            let received_rev_arg = self.content(true);

            // If the item is receiving a reverse argument, then we should
            // not update the `list_last` state
            if !received_rev_arg {
                self.list_state.store_last(current);
            }
            current = self.builder.checkpoint();
        }

        self.list_state = parent_state;
    }

    /// Parsing Helper
    /// Parse a group of items which is enclosed by a pair of tokens
    #[inline]
    fn item_group(&mut self, group_kind: SyntaxKind) {
        assert!(matches!(
            group_kind,
            ItemCurly | ItemBracket | ItemParen | ItemFormula
        ));
        // Get the corresponding closing token
        let (end_token, scope) = match group_kind {
            ItemCurly => (Token::Right(BraceKind::Curly), ParseScope::CurlyItem),
            ItemBracket => (Token::Right(BraceKind::Bracket), ParseScope::BracketItem),
            ItemParen => (Token::Right(BraceKind::Paren), ParseScope::ParenItem),
            ItemFormula => (Token::Dollar, ParseScope::Formula),
            _ => unreachable!(),
        };

        self.builder.start_node(group_kind.into());
        self.eat();
        self.item_list(scope);
        self.eat_if(end_token);
        self.builder.finish_node();
    }

    /// Item parsers
    /// Parse any syntax item, which is the entry point of the item parsers.
    ///
    /// If `not_prefer_single_char` is false, then the parser will try to parse
    /// the item as a single character if possible
    ///
    /// Returns whether the item is receiving a reverse argument
    fn content(&mut self, not_prefer_single_char: bool) -> bool {
        let Some(c) = self.peek() else {
            return false;
        };
        match c {
            Token::And
            | Token::NewLine
            | Token::LineBreak
            | Token::Whitespace
            | Token::LineComment => self.eat(),
            Token::Apostrophe => {
                self.attach_component(false);
                return true;
            }
            Token::Underline | Token::Caret => {
                self.attach_component(true);
                return true;
            }
            Token::Left(BraceKind::Curly) => self.item_group(ItemCurly),
            Token::Right(BraceKind::Curly) => {
                self.builder.start_node(TokenError.into());
                self.eat();
                self.builder.finish_node();
            }
            // todo: check if this is correct
            // self.expect2(Token::Right(BraceKind::Bracket), Token::Right(BraceKind::Paren));
            // alternative self.expect(tok);
            Token::Left(BraceKind::Bracket) if not_prefer_single_char => {
                self.item_group(ItemBracket)
            }
            Token::Left(BraceKind::Paren) if not_prefer_single_char => self.item_group(ItemParen),
            Token::Left(..) | Token::Right(..) => {
                self.eat();
            }
            Token::Word => {
                if not_prefer_single_char {
                    self.text()
                } else {
                    self.single_char();
                }
            }
            Token::Comma => self.text(),
            Token::Tilde => self.eat(),
            Token::Divide => self.eat(),
            Token::Equal => self.eat(),
            Token::Dollar => self.item_group(ItemFormula),
            Token::CommandName(name) => match name {
                CommandName::Generic => return self.command(),
                CommandName::BeginEnvironment => self.environment(),
                CommandName::EndEnvironment => return self.command(),
                CommandName::BeginBlockComment => self.block_comment(),
                CommandName::EndBlockComment => return self.command(),
                CommandName::Left => self.item_lr(),
                CommandName::Right => return self.command(),
            },
        }

        false
    }

    /// Item parsers
    /// Parse a text item
    fn text(&mut self) {
        fn is_text_component(kind: Token) -> bool {
            use Token::*;
            matches!(kind, LineBreak | Whitespace | LineComment | Word | Comma)
        }

        self.builder.start_node(ItemText.into());
        self.eat();
        while self.peek().map_or(false, is_text_component) {
            self.eat();
        }
        self.builder.finish_node();
    }

    /// Item parsers
    /// Parse a group of items which is enclosed by a pair of curly braces,
    /// but accept a word as the enclosing token
    ///
    /// Returns the word if it is present
    fn curly_group_word(&mut self) -> Option<&'a str> {
        self.builder.start_node(ItemCurly.into());
        self.eat();
        let mut w = self.lexer.peek_text();
        match self.peek() {
            Some(Token::Word | Token::CommandName(_)) => {
                self.eat_as(TokenWord);
            }
            Some(_) | None => w = None,
        }
        self.eat_if(Token::Right(BraceKind::Curly));
        self.builder.finish_node();
        w
    }

    /// Internally used by `Self::command`
    fn start_command_at(&mut self, pos: Option<Checkpoint>) {
        if let Some(available_pos) = pos {
            self.builder.start_node_at(available_pos, ItemCmd.into());
            self.builder
                .start_node_at(available_pos, ClauseArgument.into());
            self.builder.finish_node();
        } else {
            self.builder.start_node(ItemCmd.into());
        }
    }

    /// Item parsers
    /// Parse a command
    fn command(&mut self) -> bool {
        // Process a command by corresponding command specification
        // Prepare the argument matcher for succeeding parsers
        let cmd_name = self.lexer.peek_text().unwrap().strip_prefix('\\').unwrap();
        let arg_shape = self.spec.get_cmd(cmd_name).map(|cmd| &cmd.args);
        let (right_pat, is_infix) = match arg_shape {
            None | Some(ArgShape::Right(ArgPattern::None | ArgPattern::FixedLenTerm(0))) => {
                self.builder.start_node(ItemCmd.into());
                self.eat();
                self.builder.finish_node();
                return false;
            }
            Some(ArgShape::Left1) => {
                // Wrap previous item
                self.start_command_at(self.list_last().or(self.list_start()));
                self.eat();
                self.builder.finish_node();
                return true;
            }
            Some(ArgShape::Right(pattern)) => {
                self.builder.start_node(ItemCmd.into());
                (pattern, false)
            }
            Some(ArgShape::InfixGreedy) => {
                // Wrap all previous items in the scope of list
                let pos = self.list_state.take_start();
                self.start_command_at(pos);
                (&ArgPattern::Greedy, true)
            }
        };
        let searcher = self.arg_matchers.start_match(right_pat);

        self.eat();

        if is_infix {
            self.builder.start_node(ClauseArgument.into());
            self.match_arguments::<false>(searcher);
            self.builder.finish_node();
        } else {
            self.match_arguments::<true>(searcher);
        }

        self.builder.finish_node();

        is_infix
    }

    /// Item parsers
    /// Parse an environment
    fn environment(&mut self) {
        self.builder.start_node(ItemEnv.into());

        // environment begin
        {
            self.builder.start_node(ItemBegin.into());

            self.eat();
            self.trivia();

            let env_name = (self.peek() == Some(Token::Left(BraceKind::Curly)))
                .then(|| self.curly_group_word())
                .flatten();
            let arg_shape = env_name.and_then(|tok| self.spec.get_env(tok));
            let right_pat = match arg_shape.map(|cmd| &cmd.args) {
                None | Some(ArgPattern::None | ArgPattern::FixedLenTerm(0)) => None,
                Some(pattern) => Some(pattern),
            };
            let searcher = right_pat.map(|right_pat| self.arg_matchers.start_match(right_pat));

            if let Some(searcher) = searcher {
                self.match_arguments::<true>(searcher);
            }

            self.builder.finish_node();
        }

        self.item_list(ParseScope::Environment);

        if self.peek() == Some(Token::CommandName(CommandName::EndEnvironment)) {
            self.builder.start_node(ItemEnd.into());
            self.eat();
            self.trivia();

            if self.peek() == Some(Token::Left(BraceKind::Curly)) {
                self.curly_group_word();
            }

            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    /// Item parsers
    /// Parse an environment with left and right delimiters
    #[inline]
    fn item_lr(&mut self) {
        self.builder.start_node(ItemLR.into());
        self.clause_lr();

        self.item_list(ParseScope::LR);

        if self.peek() == Some(Token::CommandName(CommandName::Right)) {
            self.clause_lr();
        }

        self.builder.finish_node();
    }

    /// Item parsers
    /// Parse a block comment
    fn block_comment(&mut self) {
        self.builder.start_node(ItemBlockComment.into());
        self.eat();

        while let Some(kind) = self.peek() {
            match kind {
                Token::CommandName(CommandName::BeginBlockComment) => {
                    self.block_comment();
                }
                Token::CommandName(CommandName::EndBlockComment) => {
                    self.eat();
                    break;
                }
                _ => {
                    self.eat();
                }
            }
        }

        self.builder.finish_node();
    }

    /// Clause parsers
    /// Parse the arguments of a command
    ///
    /// It feeds the argument matcher with encoded argument kinds
    /// - Bracket/b: []
    /// - Parenthesis/p: ()
    /// - Term/t: any rest of terms, typically {} or single char
    #[inline]
    fn match_arguments_<const WRAP_ARGS: bool>(&mut self, mut searcher: ArgMatcher) {
        // const NOT_INFIX = WRAP_ARGS

        fn arg<'a, const WRAP_ARGS: bool, T>(
            this: &mut Parser<'a>,
            f: impl FnOnce(&mut Parser<'a>) -> T,
        ) -> T {
            if WRAP_ARGS {
                this.builder.start_node(ClauseArgument.into());
                let res = f(this);
                this.builder.finish_node();

                res
            } else {
                f(this)
            }
        }

        let mut current = if !WRAP_ARGS {
            Some(self.builder.checkpoint())
        } else {
            None
        };
        while let Some(kind) = self.peek() {
            match kind {
                // trivials
                Token::LineBreak | Token::Whitespace | Token::LineComment => self.eat(),
                // Argument matches is stopped on these tokens anyway
                Token::And | Token::NewLine => return,
                // WRAP_ARGS also determines whether it could be regards as an attachment.
                Token::Caret | Token::Underline if WRAP_ARGS => {
                    return;
                }
                // prefer rob characters from words as arguments
                Token::Word if !searcher.is_greedy() => {
                    // Split the word into single characters for term matching
                    let mut split_cnt = 0usize;
                    for c in self.lexer.peek_text().unwrap().chars() {
                        if !searcher.try_match(ARGUMENT_KIND_TERM) {
                            if split_cnt > 0 {
                                self.lexer.consume_word(split_cnt);
                            }
                            return;
                        }
                        split_cnt += 1;

                        arg::<WRAP_ARGS, _>(self, |this| {
                            this.builder.token(TokenWord.into(), &c.to_string())
                        });
                    }

                    if !WRAP_ARGS {
                        // If consumed to end, this is right
                        // Otherwise, whether it is right does not matter
                        current = Some(self.builder.checkpoint());
                    }
                    // Consume part of the word
                    if split_cnt > 0 {
                        self.lexer.consume_word(split_cnt);
                    }
                    if !WRAP_ARGS {
                        self.list_state.may_store_last(current);
                    }
                }
                Token::Left(bk) => {
                    let (encoded, sk) = match bk {
                        BraceKind::Curly => (ARGUMENT_KIND_TERM, ItemCurly),
                        BraceKind::Bracket => (ARGUMENT_KIND_BRACKET, ItemBracket),
                        BraceKind::Paren => (ARGUMENT_KIND_PAREN, ItemParen),
                    };

                    let Some(modified_as_term) = searcher.match_as_term(encoded) else {
                        return;
                    };

                    if !WRAP_ARGS {
                        // If consumed to end, this is right
                        // Otherwise, whether it is right does not matter
                        current = Some(self.builder.checkpoint());
                    }
                    arg::<WRAP_ARGS, _>(self, |this| {
                        if modified_as_term {
                            this.eat();
                        } else {
                            this.item_group(sk);
                        }
                    });

                    if !WRAP_ARGS {
                        self.list_state.may_store_last(current);
                    }
                }
                // rest of any item
                kind => {
                    if self.stop_by_scope(kind) || !searcher.try_match(ARGUMENT_KIND_TERM) {
                        return;
                    }

                    if !WRAP_ARGS {
                        // If consumed to end, this is right
                        // Otherwise, whether it is right does not matter
                        current = Some(self.builder.checkpoint());
                    }
                    let has_rev_argument = arg::<WRAP_ARGS, _>(self, |this| this.content(true));
                    if !WRAP_ARGS && !has_rev_argument {
                        self.list_state.may_store_last(current);
                    }
                }
            }
        }
    }

    #[inline]
    fn match_arguments<const NOT_INFIX: bool>(&mut self, searcher: ArgMatcher) {
        self.list_state.may_store_last(None);
        let last = self.list_last();
        let start = if NOT_INFIX {
            self.list_state.may_store_start(None);
            self.list_start()
        } else {
            None
        };

        self.match_arguments_::<NOT_INFIX>(searcher);

        if NOT_INFIX {
            self.list_state.may_store_start(start);
        }
        self.list_state.may_store_last(last);
    }

    /// Clause parsers
    /// Parse a component
    fn attach_component(&mut self, has_script: bool) {
        if let Some(list_last) = self.list_last() {
            self.builder
                .start_node_at(list_last, ItemAttachComponent.into());
            self.builder.start_node_at(list_last, ClauseArgument.into());
            self.builder.finish_node();
        } else if !has_script {
            self.eat();
            return;
        } else {
            self.builder.start_node(ItemAttachComponent.into());
        }

        self.eat();

        if has_script {
            self.trivia();
            self.content(false);
        }

        self.builder.finish_node();
    }

    /// Clause parsers
    /// Parse a left or right delimiter
    fn clause_lr(&mut self) {
        self.builder.start_node(ClauseLR.into());

        self.eat();
        self.trivia();
        match self.peek() {
            Some(Token::CommandName(CommandName::Generic)) => {
                self.eat_as(TokenCommandSym);
            }
            // invalid syntax
            Some(Token::CommandName(..) | Token::Dollar) | None => {}
            Some(Token::Word) => {
                self.single_char();
            }
            _ => self.eat(),
        }

        self.builder.finish_node();
    }
}

/// Parse the input text with the given command specification
/// and return the untyped syntax tree
///
/// The error nodes are attached to the tree
pub fn parse(input: &str, spec: CommandSpec) -> SyntaxNode {
    SyntaxNode::new_root(Parser::new(input, spec).parse())
}
