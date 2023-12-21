mod arg_match;
pub mod parser;
pub mod syntax;

pub use parser::parse;

pub use mitex_spec as spec;
pub use spec::preludes::command as command_preludes;
pub use spec::*;
