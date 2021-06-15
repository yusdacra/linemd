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

/// Renders parsed tokens (`Vec<Token>`) as HTML.
///
/// # Example
/// ```
/// # use linemd::{parse, render_as_html};
/// let html = render_as_html(parse("Some uninspiring text."));
/// ```
#[cfg(feature = "html")]
pub fn render_as_html(tokens: alloc::vec::Vec<Token>) -> String {
    let mut html = String::new();

    let mut unordered_list = false;
    let mut ordered_list = false;

    let mut was_line_break = false;

    for token in tokens {
        let is_unordered_item = matches!(
            token,
            Token::ListItem {
                place: None,
                text: _
            }
        );
        let ordered_item = matches!(
            token,
            Token::ListItem {
                place: Some(_),
                text: _
            }
        );
        let is_line_break = matches!(token, Token::LineBreak);

        if !unordered_list && is_unordered_item {
            html.push_str("<ul>\n");
            unordered_list = true;
        } else if (was_line_break || !is_line_break)
            && (!is_unordered_item || is_line_break)
            && unordered_list
        {
            html.push_str("</ul>\n");
            unordered_list = false;
        }

        if !ordered_list && ordered_item {
            html.push_str("<ol>\n");
            ordered_list = true;
        } else if (was_line_break || !is_line_break)
            && (!ordered_item || is_line_break)
            && ordered_list
        {
            html.push_str("</ol>\n");
            ordered_list = false;
        }

        let tok_html = token.into_html();
        html.push_str(&tok_html);

        was_line_break = is_line_break;
    }

    html.trim().to_string()
}
