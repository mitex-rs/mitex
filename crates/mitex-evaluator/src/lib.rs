use std::collections::HashMap;

use mitex_lexer::{Lexer, Token};
use mitex_spec::CommandSpec;

pub type Snapshot = ();

/// MacroEvaluator has exact same interface as Lexer, but it expands macros.
///
/// When it meets a macro in token stream, It evaluates a macro into expanded
/// tokens.
pub struct MacroEvaluator<'a> {
    /// Lexer level structure
    lexer: Lexer<'a>,
    /// Scoped unified symbol table
    symbol_table: HashMap<String, String>,
}

impl<'a> MacroEvaluator<'a> {
    /// Create a new macro evaluator
    pub fn new(input: &'a str, spec: CommandSpec) -> Self {
        Self {
            lexer: Lexer::new(input, spec),
            symbol_table: HashMap::new(),
        }
    }

    /// Peek the next token
    pub fn peek(&self) -> Option<Token> {
        self.lexer.peek()
    }

    /// Create a new scope for macro definitions
    pub fn create_scope(&mut self) -> Snapshot {}

    /// Restore the scope (delete all macros defined in the child scope)
    pub fn restore(&mut self, _snapshot: Snapshot) {}

    /// Peek the next token and its text
    pub fn add_macro(&mut self, name: &str, value: &str) {
        self.symbol_table.insert(name.to_owned(), value.to_owned());
    }
}
