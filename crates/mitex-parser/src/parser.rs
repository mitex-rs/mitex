use rowan::{Checkpoint, GreenNode, GreenNodeBuilder};

use crate::arg_match::{ArgMatcher, ArgMatcherBuilder};
use crate::spec::argument_kind::*;
use crate::syntax::SyntaxKind::{self, *};
use crate::{ArgPattern, ArgShape, CommandSpec};
use mitex_lexer::{BraceKind, CommandName, IfCommandName, Lexer, MacroEngine, Token, TokenStream};

/// Stacked scope for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseScope {
    /// The root scope, this is set when the parser enters the entry point
    Root,
    /// The scope of a formula, i.e. `$...$` or `$$...$$`
    DollarFormula,
    /// The scope of a formula, i.e. `\(..\)` or `\[..\]`
    CmdFormula,
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
    use rowan::Checkpoint;

    use super::ParseScope;

    /// State used by list parsers
    #[derive(Debug, Clone, Copy)]
    pub struct ListState {
        /// The checkpoint of the first item in the list
        /// Note: if an infix command is parsed, this will become None
        start: Option<Checkpoint>,
        /// The checkpoint of the last item in the list
        last: Option<Checkpoint>,
        /// The current scope
        pub scope: ParseScope,
    }

    impl Default for ListState {
        fn default() -> Self {
            Self {
                start: None,
                last: None,
                scope: ParseScope::Root,
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
            self.start
        }

        /// The last position of the list
        #[inline]
        pub fn last(&self) -> Option<Checkpoint> {
            self.last
        }

        /// Take the start position of the list
        #[inline]
        pub fn take_start(&mut self) -> Option<Checkpoint> {
            self.start.take()
        }

        /// Store the start position of the list
        pub fn store_start(&mut self, current: Checkpoint) {
            self.start = Some(current);
        }

        /// Store the last position of the list
        pub fn store_last(&mut self, current: Checkpoint) {
            self.last = Some(current);
        }

        /// Store the last position of the list
        pub fn may_store_start(&mut self, current: Option<Checkpoint>) {
            self.start = current;
        }

        /// Store the last position of the list
        pub fn may_store_last(&mut self, current: Option<Checkpoint>) {
            self.last = current;
        }
    }
}
use list_state::ListState;

/// The mutable parser that parse the input text into a syntax tree
#[derive(Debug)]
pub struct Parser<'a, S: TokenStream<'a> = ()> {
    /// Lexer level structure
    lexer: Lexer<'a, S>,
    /// Helper for building syntax tree
    builder: GreenNodeBuilder<'static>,

    /// Command specification
    spec: CommandSpec,
    /// Argument matcher builder containing cached regexes
    arg_matchers: ArgMatcherBuilder,
    /// trivia buffer
    trivia_buffer: Vec<(Token, &'a str)>,

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
            trivia_buffer: Vec::new(),
        }
    }

    /// For internal testing
    pub fn new_macro(text: &'a str, spec: CommandSpec) -> Parser<'a, MacroEngine<'a>> {
        let lexer = Lexer::new_with_bumper(text, spec.clone(), MacroEngine::new(spec.clone()));
        Parser::<'a, MacroEngine<'a>> {
            lexer,
            builder: GreenNodeBuilder::new(),
            spec,
            arg_matchers: ArgMatcherBuilder::default(),
            list_state: Default::default(),
            trivia_buffer: Vec::new(),
        }
    }
}

impl<'a, S: TokenStream<'a>> Parser<'a, S> {
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

    /// List State
    /// Whether the current scope is env
    #[inline]
    fn inside_env(&self) -> bool {
        self.list_state.scope == ParseScope::Environment
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
    /// Drop the next token
    fn drop(&mut self) {
        self.lexer.eat();
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
        }
    }

    /// Lexer Interface
    /// Hold the next trivia token
    fn hold_trivia(&mut self) {
        self.trivia_buffer.push(self.lexer.eat().unwrap());
    }

    /// Lexer Interface
    fn ignore_holding_trivia(&mut self) {
        self.trivia_buffer.clear();
    }

    /// Lexer Interface
    fn extract_holding_trivia(&mut self) {
        for (kind, text) in self.trivia_buffer.drain(..) {
            let kind: SyntaxKind = kind.into();
            self.builder.token(kind.into(), text);
        }
    }

    /// Lexer Interface
    fn single_char(&mut self) -> Option<()> {
        let first_char = self.lexer.peek_char()?;
        self.builder
            .token(TokenWord.into(), &first_char.to_string());
        self.lexer.consume_utf8_bytes(first_char.len_utf8());

        Some(())
    }

    /// Lexer Interface
    /// Consume tokens until the next non-trivia token
    fn trivia(&mut self) {
        while self.peek().as_ref().map_or(false, Token::is_trivia) {
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
            ParseScope::DollarFormula => matches!(
                kind,
                Token::Right(BraceKind::Curly)
                    | Token::CommandName(CommandName::EndEnvironment | CommandName::Right)
                    | Token::Dollar
            ),
            ParseScope::CmdFormula => matches!(
                kind,
                Token::Right(BraceKind::Curly)
                    | Token::CommandName(
                        CommandName::EndEnvironment | CommandName::Right | CommandName::EndMath
                    )
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
            let attachable = self.content(true);

            // If the item is not attachable, then we should
            // not update the `list_last` state
            if attachable {
                self.list_state.store_last(current);
            }
            current = self.builder.checkpoint();
        }

        self.list_state = parent_state;
    }

    /// Parsing Helper
    /// Parse a group of items which is enclosed by a pair of tokens
    #[inline]
    fn item_group(&mut self, scope: ParseScope) {
        assert!(matches!(
            scope,
            ParseScope::CurlyItem
                | ParseScope::BracketItem
                | ParseScope::ParenItem
                | ParseScope::DollarFormula
                | ParseScope::CmdFormula,
        ));
        // Get the corresponding closing token
        let (end_token, group_kind) = match scope {
            ParseScope::CurlyItem => (Token::Right(BraceKind::Curly), ItemCurly),
            ParseScope::BracketItem => (Token::Right(BraceKind::Bracket), ItemBracket),
            ParseScope::ParenItem => (Token::Right(BraceKind::Paren), ItemParen),
            ParseScope::DollarFormula => (Token::Dollar, ItemFormula),
            ParseScope::CmdFormula => (Token::CommandName(CommandName::EndMath), ItemFormula),
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
    /// Returns whether the item is attachable
    fn content(&mut self, not_prefer_single_char: bool) -> bool {
        let Some(c) = self.peek() else {
            return true;
        };
        match c {
            Token::Ampersand
            | Token::NewLine
            | Token::LineBreak
            | Token::Whitespace
            | Token::LineComment
            | Token::Hash
            | Token::Asterisk
            | Token::AtSign
            | Token::Error => {
                self.eat();
                return false;
            }
            Token::Apostrophe => {
                self.attach_component(false);
                return false;
            }
            Token::Underscore | Token::Caret => {
                self.attach_component(true);
                return false;
            }
            Token::Left(BraceKind::Curly) => self.item_group(ParseScope::CurlyItem),
            Token::Right(BraceKind::Curly) | Token::MacroArg(_) => {
                self.builder.start_node(TokenError.into());
                self.eat();
                self.builder.finish_node();
            }
            Token::Left(..)
            | Token::Right(..)
            | Token::Tilde
            | Token::Slash
            | Token::Ditto
            | Token::Semicolon => self.eat(),
            Token::Word => {
                if not_prefer_single_char {
                    self.text()
                } else {
                    self.single_char();
                }
            }
            Token::Comma => self.text(),
            Token::Dollar => {
                self.item_group(ParseScope::DollarFormula);
                return false;
            }
            Token::CommandName(name) => match name {
                CommandName::Generic => return self.command(),
                CommandName::BeginEnvironment => self.environment(),
                CommandName::BeginMath => {
                    self.item_group(ParseScope::CmdFormula);
                    return false;
                }
                CommandName::If(IfCommandName::IfFalse) => self.block_comment(),
                CommandName::If(IfCommandName::IfTypst) => self.typst_code(),
                CommandName::If(..) | CommandName::Else | CommandName::EndIf => {
                    return self.command()
                }
                CommandName::Left => self.item_lr(),
                CommandName::Right => return self.command(),
                CommandName::ErrorBeginEnvironment | CommandName::ErrorEndEnvironment => self.eat(),
                // todo: raise error end environment
                //
                // See:
                //
                // ```plain
                // assert_debug_snapshot!(parse(r#"\end{}"#), @r###"
                // root
                // |error'(sym'(""))
                // "###);
                // ```
                CommandName::EndEnvironment | CommandName::EndMath => {
                    self.builder.start_node(TokenError.into());
                    self.eat();
                    self.builder.finish_node();
                }
            },
        }

        true
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
        let right_pat = match arg_shape {
            None
            | Some(ArgShape::Right {
                pattern: ArgPattern::None | ArgPattern::FixedLenTerm { len: 0 },
            }) => {
                self.builder.start_node(ItemCmd.into());
                self.eat();
                self.builder.finish_node();
                return true;
            }
            Some(ArgShape::Left1) => {
                // Wrap previous item
                self.start_command_at(self.list_last().or(self.list_start()));
                self.eat();
                self.builder.finish_node();
                return false;
            }
            Some(ArgShape::Right { pattern }) => {
                self.builder.start_node(ItemCmd.into());
                pattern
            }
            Some(ArgShape::InfixGreedy) => {
                // Wrap all previous items in the scope of list
                let pos = self.list_state.take_start();
                self.start_command_at(pos);
                &ArgPattern::Greedy
            }
        };
        let searcher = self.arg_matchers.start_match(right_pat);
        let is_greedy = searcher.is_greedy();

        self.eat();

        if is_greedy {
            self.builder.start_node(ClauseArgument.into());
            self.match_arguments::<true>(searcher);
            self.builder.finish_node();
        } else {
            self.match_arguments::<false>(searcher);
        }

        self.builder.finish_node();

        self.extract_holding_trivia();

        !is_greedy
    }

    /// Item parsers
    /// Parse an environment
    fn environment(&mut self) {
        self.builder.start_node(ItemEnv.into());

        // environment begin
        {
            self.builder.start_node(ItemBegin.into());

            let env_name = self.lexer.peek_text().unwrap();
            self.eat();

            let arg_shape = self.spec.get_env(env_name);
            let right_pat = match arg_shape.map(|cmd| &cmd.args) {
                None | Some(ArgPattern::None | ArgPattern::FixedLenTerm { len: 0 }) => None,
                Some(pattern) => Some(pattern),
            };
            let searcher = right_pat.map(|right_pat| self.arg_matchers.start_match(right_pat));

            if let Some(searcher) = searcher {
                self.match_arguments::<false>(searcher);
            }

            self.builder.finish_node();

            self.extract_holding_trivia();
        }

        self.item_list(ParseScope::Environment);

        if self.peek() == Some(Token::CommandName(CommandName::EndEnvironment)) {
            self.builder.start_node(ItemEnd.into());
            self.eat();
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
        self.drop();
        self.eat_body_of_ifs();
        self.builder.finish_node();
    }

    fn typst_code(&mut self) {
        self.builder.start_node(ItemTypstCode.into());
        self.drop();
        self.eat_body_of_ifs();
        self.builder.finish_node();
    }

    fn eat_body_of_ifs(&mut self) {
        let mut nested = 0;
        while let Some(kind) = self.peek() {
            match kind {
                Token::CommandName(CommandName::If(..)) => {
                    // todo: nest block comment
                    // self.block_comment();
                    self.eat();
                    nested += 1;
                }
                Token::CommandName(CommandName::EndIf) => {
                    if nested == 0 {
                        self.drop();
                        break;
                    }
                    self.eat();
                    nested -= 1;
                }
                _ => {
                    self.eat();
                }
            }
        }
    }

    /// Clause parsers
    /// Parse the arguments of a command
    ///
    /// It feeds the argument matcher with encoded argument kinds
    /// - Bracket/b: []
    /// - Parenthesis/p: ()
    /// - Term/t: any rest of terms, typically {} or single char
    #[inline]
    fn match_arguments_<const GREEDY: bool>(&mut self, mut searcher: ArgMatcher) {
        assert!((GREEDY == searcher.is_greedy()), "GREEDY mismatched");

        // const WRAP_ARGS = !GREEDY
        macro_rules! k_wrap_args {
            () => {
                !GREEDY
            };
        }

        fn arg<'a, const GREEDY: bool, T, S: TokenStream<'a>>(
            this: &mut Parser<'a, S>,
            f: impl FnOnce(&mut Parser<'a, S>) -> T,
        ) -> T {
            if k_wrap_args!() {
                this.ignore_holding_trivia();
                this.builder.start_node(ClauseArgument.into());
                let res = f(this);
                this.builder.finish_node();

                res
            } else {
                f(this)
            }
        }

        let mut current = if !k_wrap_args!() {
            Some(self.builder.checkpoint())
        } else {
            None
        };
        while let Some(kind) = self.peek() {
            match kind {
                // trivials
                Token::LineBreak | Token::Whitespace | Token::LineComment => {
                    if GREEDY {
                        self.eat();
                    } else {
                        self.hold_trivia();
                    }
                }
                // Argument matches is stopped on these tokens
                // However, newline is also a command (with name `\`), hence this is different from
                // mark and (`&`)
                //
                // Condition explained.
                // If it is a greedy command/operator, i.e. GREEDY,
                //   stops only if parser is inside of some environment
                //   e.g. (stops) \begin{matrix} \displaystyle 1 \\ 3 \\ \end{matrix}
                //   e.g. (don't stops) \displaystyle \frac{1}{2} \\ \frac{1}{2}
                //   e.g. (don't stops) \left. \displaystyle \frac{1}{2} \\ \frac{1}{2} \right.
                // Othersise, it is a regular command,
                //   treated as a command (with name `\`) first.
                //   e.g.(don't stops) \begin{matrix}\frac{1} \\ {2}\end{matrix}
                Token::NewLine if GREEDY && self.inside_env() => return,
                // Argument matches is stopped on these tokens anyway
                Token::Ampersand => return,
                // k_wrap_args!() also determines whether it could be regards as an attachment.
                Token::Caret | Token::Underscore if k_wrap_args!() => {
                    return;
                }
                // prefer rob characters from words as arguments
                Token::Word if !GREEDY => {
                    // Split the word into single characters for term matching
                    let mut split_cnt = 0usize;
                    for c in self.lexer.peek_text().unwrap().chars() {
                        if !searcher.try_match(ARGUMENT_KIND_TERM) {
                            if split_cnt > 0 {
                                self.lexer.consume_utf8_bytes(split_cnt);
                            }
                            return;
                        }
                        split_cnt += c.len_utf8();

                        arg::<GREEDY, _, _>(self, |this| {
                            this.builder.token(TokenWord.into(), &c.to_string())
                        });
                    }

                    if !k_wrap_args!() {
                        // If consumed to end, this is right
                        // Otherwise, whether it is right does not matter
                        current = Some(self.builder.checkpoint());
                    }
                    // Consume part of the word
                    if split_cnt > 0 {
                        self.lexer.consume_utf8_bytes(split_cnt);
                    }
                    if !k_wrap_args!() {
                        self.list_state.may_store_last(current);
                    }
                }
                Token::Left(bk) => {
                    let (encoded, scope) = match bk {
                        BraceKind::Curly => (ARGUMENT_KIND_TERM, ParseScope::CurlyItem),
                        BraceKind::Bracket => (ARGUMENT_KIND_BRACKET, ParseScope::BracketItem),
                        BraceKind::Paren => (ARGUMENT_KIND_PAREN, ParseScope::ParenItem),
                    };

                    let Some(modified_as_term) = searcher.match_as_term(encoded) else {
                        return;
                    };

                    if !k_wrap_args!() {
                        // If consumed to end, this is right
                        // Otherwise, whether it is right does not matter
                        current = Some(self.builder.checkpoint());
                    }
                    arg::<GREEDY, _, _>(self, |this| {
                        if modified_as_term {
                            this.eat();
                        } else {
                            this.item_group(scope);
                        }
                    });

                    if !k_wrap_args!() {
                        self.list_state.may_store_last(current);
                    }
                }
                // rest of any item
                kind => {
                    if self.stop_by_scope(kind) || !searcher.try_match(ARGUMENT_KIND_TERM) {
                        return;
                    }

                    if !k_wrap_args!() {
                        // If consumed to end, this is right
                        // Otherwise, whether it is right does not matter
                        current = Some(self.builder.checkpoint());
                    }
                    let attachable = arg::<GREEDY, _, _>(self, |this| this.content(true));
                    if !k_wrap_args!() && attachable {
                        self.list_state.may_store_last(current);
                    }
                }
            }
        }
    }

    #[inline]
    fn match_arguments<const GREEDY: bool>(&mut self, searcher: ArgMatcher) {
        self.list_state.may_store_last(None);
        let last = self.list_last();
        let start = self.list_start();
        self.list_state
            .may_store_start(GREEDY.then(|| self.builder.checkpoint()));

        self.match_arguments_::<GREEDY>(searcher);

        self.list_state.may_store_start(start);
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
