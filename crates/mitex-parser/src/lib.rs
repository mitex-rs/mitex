//! Given source strings, MiTeX Parser provides an AST (abstract syntax tree).
//!
//! ## Option: Command Specification
//! The parser retrieves a command specification which defines shape of
//! commands. With the specification, the parser can parse commands correctly.
//! Otherwise, all commands are parsed as barely names without arguments.
//!
//! ## Produce: AST
//! It returns an untyped syntax node representing the AST defined by [`rowan`].
//! You can access the AST conveniently with interfaces provided by
//! [`rowan::SyntaxNode`].
//!
//! The untyped syntax node can convert to typed ones defined in
//! [`crate::syntax`].
//!
//! The untyped syntax node can also convert to [`rowan::cursor::SyntaxNode`] to
//! modify the AST syntactically.

mod arg_match;
mod parser;
pub mod syntax;

pub use mitex_spec as spec;
pub use spec::preludes::command as command_preludes;
pub use spec::*;
use syntax::SyntaxNode;

use parser::Parser;

/// Parse the input text with the given command specification
/// and return the untyped syntax tree
///
/// The error nodes are attached to the tree
pub fn parse(input: &str, spec: CommandSpec) -> SyntaxNode {
    SyntaxNode::new_root(Parser::new_macro(input, spec).parse())
}

/// It is only for internal testing
pub fn parse_without_macro(input: &str, spec: CommandSpec) -> SyntaxNode {
    SyntaxNode::new_root(Parser::new(input, spec).parse())
}
