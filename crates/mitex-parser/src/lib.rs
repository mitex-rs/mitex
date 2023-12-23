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
