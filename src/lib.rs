pub mod parser;
#[cfg(test)]
mod tests;

pub use parser::{Parser, Token};

pub fn parse(md: impl AsRef<str>) -> Vec<Token> {
    {
        let mut parser = Parser::new();
        parser.feed(md);
        parser
    }
    .parse()
}
