//! `linemd` is a simple and opinionated markdown parsing library.

mod parser;
#[cfg(test)]
mod tests;

#[doc(inline)]
pub use parser::{Parser, Token};

/// Parses markdown text and returns parsed tokens.
///
/// # Example
/// ```
/// # use linemd::Token;
/// let tokens = linemd::parse("Some uninspiring text.");
/// // Use the tokens
/// # assert_eq!(tokens, vec![Token::Text { value: "Some uninspiring text.".to_string(), bold: false, italic: false }]);
/// ```
pub fn parse(md: impl AsRef<str>) -> Vec<Token> {
    {
        let mut parser = Parser::new();
        parser.feed(md.as_ref());
        parser
    }
    .parse()
}
