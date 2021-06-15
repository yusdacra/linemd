#![no_std]
//! `linemd` is a simple and opinionated markdown parsing library.

extern crate alloc;
#[cfg(feature = "html")]
use alloc::format;
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

mod parser;
#[cfg(test)]
mod tests;

#[cfg(feature = "html")]
mod html;
#[cfg(feature = "svg")]
mod svg;

#[doc(inline)]
pub use parser::{Parser, Token};

#[cfg(feature = "svg")]
#[doc(inline)]
pub use svg::render_as_svg;

#[cfg(feature = "html")]
#[doc(inline)]
pub use html::render_as_html;

/// Parses markdown text and returns parsed tokens.
///
/// # Example
/// ```
/// # use linemd::Token;
/// let tokens = linemd::parse("Some uninspiring text.");
/// // Use the tokens
/// assert_eq!(tokens, vec![Token::Text { value: "Some uninspiring text.".to_string(), bold: false, italic: false }]);
/// ```
pub fn parse(md: impl AsRef<str>) -> Vec<Token> {
    {
        let mut parser = Parser::new();
        parser.feed(md.as_ref());
        parser
    }
    .parse()
}
