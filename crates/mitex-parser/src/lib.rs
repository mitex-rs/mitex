mod lexer;
pub mod parser;
pub mod syntax;

pub use parser::parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy() {
        println!(
            "{:#?}",
            parse(
                r#"Foo
\iffalse
Test1
\fi
Bar
\iffalse
\fii
\fi
Baz"#
            )
        );
    }
}
