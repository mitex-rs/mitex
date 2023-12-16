use crate::syntax::SyntaxKind::*;
use rowan::{GreenNode, GreenNodeBuilder};

use crate::syntax::SyntaxNode;

use crate::lexer::{CommandName, Lexer, Token};

#[derive(Debug, Clone, Copy)]
struct ParserContext {
    allow_environment: bool,
    allow_comma: bool,
}

impl Default for ParserContext {
    fn default() -> Self {
        Self {
            allow_environment: true,
            allow_comma: true,
        }
    }
}

#[derive(Debug)]
struct Parser<'a> {
    lexer: Lexer<'a>,
    builder: GreenNodeBuilder<'static>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(text),
            builder: GreenNodeBuilder::new(),
        }
    }

    fn eat(&mut self) {
        let (kind, text) = self.lexer.eat().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn peek(&self) -> Option<Token> {
        self.lexer.peek()
    }

    fn expect(&mut self, kind: Token) {
        if self.peek() == Some(kind) {
            self.eat();
            self.trivia();
        }
    }

    fn expect2(&mut self, kind1: Token, kind2: Token) {
        if self
            .peek()
            .filter(|&kind| kind == kind1 || kind == kind2)
            .is_some()
        {
            self.eat();
            self.trivia();
        }
    }

    fn trivia(&mut self) {
        while self.peek().map_or(false, |kind| {
            matches!(
                kind,
                Token::LineBreak | Token::Whitespace | Token::LineComment
            )
        }) {
            self.eat();
        }
    }

    pub fn parse(mut self) -> GreenNode {
        self.builder.start_node(ROOT.into());
        self.preamble();
        while self.peek().is_some() {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
        self.builder.finish()
    }

    fn content(&mut self, context: ParserContext) {
        match self.peek().unwrap() {
            Token::LineBreak | Token::Whitespace | Token::LineComment => self.eat(),
            Token::LCurly if context.allow_environment => self.curly_group(),
            Token::LCurly => self.curly_group_without_environments(),
            Token::LBrack | Token::LParen => self.mixed_group(),
            Token::RCurly | Token::RBrack | Token::RParen => {
                self.builder.start_node(ERROR.into());
                self.eat();
                self.builder.finish_node();
            }
            Token::Pipe | Token::Word | Token::Comma => self.text(context),
            Token::Eq => self.eat(),
            Token::Dollar => self.formula(),
            Token::CommandName(name) => match name {
                CommandName::Generic => self.generic_command(),
                CommandName::BeginEnvironment if context.allow_environment => self.environment(),
                CommandName::BeginEnvironment => self.generic_command(),
                CommandName::EndEnvironment => self.generic_command(),
                CommandName::BeginEquation => self.equation(),
                CommandName::EndEquation => self.generic_command(),
                CommandName::MathOperator => self.math_operator(),
                CommandName::ColorReference => self.color_reference(),
                CommandName::BeginBlockComment => self.block_comment(),
                CommandName::EndBlockComment => self.generic_command(),
            },
        }
    }

    fn text(&mut self, context: ParserContext) {
        self.builder.start_node(TEXT.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| {
                matches!(
                    kind,
                    Token::LineBreak
                        | Token::Whitespace
                        | Token::LineComment
                        | Token::Word
                        | Token::Pipe
                        | Token::Comma
                ) && (context.allow_comma || kind != Token::Comma)
            })
            .is_some()
        {
            self.eat();
        }
        self.builder.finish_node();
    }

    fn curly_group(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, Token::RCurly))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_impl(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                Token::RCurly => break,
                Token::CommandName(CommandName::BeginEnvironment) => self.begin(),
                Token::CommandName(CommandName::EndEnvironment) => self.end(),
                _ => self.content(ParserContext::default()),
            };
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_without_environments(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, Token::RCurly))
            .is_some()
        {
            self.content(ParserContext {
                allow_environment: false,
                allow_comma: true,
            });
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_word(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD.into());
        self.eat();
        self.trivia();
        match self.peek() {
            Some(Token::Word | Token::Pipe) => {
                self.key();
            }
            Some(Token::CommandName(_)) => {
                self.content(ParserContext::default());
            }
            Some(_) | None => {}
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_command(&mut self) {
        self.builder.start_node(CURLY_GROUP_COMMAND.into());
        self.eat();
        self.trivia();
        if matches!(self.peek(), Some(Token::CommandName(_))) {
            self.eat();
            self.trivia();
        }

        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn brack_group(&mut self) {
        self.builder.start_node(BRACK_GROUP.into());
        self.eat();
        while self.peek().map_or(false, |kind| {
            !matches!(
                kind,
                Token::RCurly | Token::RBrack | Token::CommandName(CommandName::EndEnvironment)
            )
        }) {
            self.content(ParserContext::default());
        }

        self.expect(Token::RBrack);
        self.builder.finish_node();
    }

    fn mixed_group(&mut self) {
        self.builder.start_node(MIXED_GROUP.into());
        self.eat();
        self.trivia();
        while self.peek().map_or(false, |kind| {
            !matches!(
                kind,
                Token::RCurly
                    | Token::RBrack
                    | Token::RParen
                    | Token::CommandName(CommandName::EndEnvironment)
            )
        }) {
            self.content(ParserContext::default());
        }

        self.expect2(Token::RBrack, Token::RParen);
        self.builder.finish_node();
    }

    fn key(&mut self) {
        self.key_with_eq(true);
    }

    fn key_with_eq(&mut self, allow_eq: bool) {
        self.builder.start_node(KEY.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                Token::Whitespace | Token::LineComment | Token::Word | Token::Pipe => self.eat(),
                Token::Eq if allow_eq => self.eat(),
                _ => break,
            }
        }

        self.trivia();
        self.builder.finish_node();
    }

    fn formula(&mut self) {
        self.builder.start_node(FORMULA.into());
        self.eat();
        self.trivia();
        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    Token::RCurly | Token::CommandName(CommandName::EndEnvironment) | Token::Dollar
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(Token::Dollar);
        self.builder.finish_node();
    }

    fn generic_command(&mut self) {
        self.builder.start_node(GENERIC_COMMAND.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                Token::LineBreak | Token::Whitespace | Token::LineComment => self.eat(),
                Token::LCurly => self.curly_group(),
                Token::LBrack | Token::LParen => self.mixed_group(),
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn equation(&mut self) {
        self.builder.start_node(EQUATION.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    Token::CommandName(CommandName::EndEnvironment)
                        | Token::RCurly
                        | Token::CommandName(CommandName::EndEquation)
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(Token::CommandName(CommandName::EndEquation));
        self.builder.finish_node();
    }

    fn begin(&mut self) {
        self.builder.start_node(BEGIN.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.peek() == Some(Token::LBrack) {
            self.brack_group();
        }
        self.builder.finish_node();
    }

    fn end(&mut self) {
        self.builder.start_node(END.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }

    fn environment(&mut self) {
        self.builder.start_node(ENVIRONMENT.into());
        self.begin();

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    Token::RCurly | Token::CommandName(CommandName::EndEnvironment)
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }

        if self.peek() == Some(Token::CommandName(CommandName::EndEnvironment)) {
            self.end();
        }

        self.builder.finish_node();
    }

    fn preamble(&mut self) {
        self.builder.start_node(PREAMBLE.into());
        while self
            .peek()
            .filter(|&kind| kind != Token::CommandName(CommandName::EndEnvironment))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn block_comment(&mut self) {
        self.builder.start_node(BLOCK_COMMENT.into());
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

    fn math_operator(&mut self) {
        self.builder.start_node(MATH_OPERATOR.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_command();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_impl();
        }

        self.builder.finish_node();
    }

    fn color_reference(&mut self) {
        self.builder.start_node(COLOR_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }
}

pub fn parse(input: &str) -> SyntaxNode {
    SyntaxNode::new_root(Parser::new(input).parse())
}
