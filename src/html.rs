use crate::parser::{Text, Token};

use super::*;
use core::fmt::Write;

/// Renders parsed tokens as HTML.
///
/// # Example
/// ```
/// # use linemd::{render_as_html, Parser};
/// let html = render_as_html("Some uninspiring text.".parse_md());
/// ```
pub fn render_as_html<'a>(tokens: impl AsRef<[Token<'a>]> + 'a) -> String {
    let mut buf = String::new();
    render_to_buffer(tokens, &mut buf);
    buf
}

/// Renders parsed tokens as HTML, to a buffer.
///
/// # Example
/// ```
/// # use linemd::{html, Parser};
/// let mut buffer = String::new();
/// let html = html::render_to_buffer("Some uninspiring text.".parse_md(), &mut buffer);
/// ```
pub fn render_to_buffer<'a>(tokens: impl AsRef<[Token<'a>]> + 'a, buf: &mut String) {
    let mut in_unordered_list = false;
    let mut in_ordered_list = false;

    let mut was_line_break = false;
    let mut in_paragraph = false;

    let tokens = tokens.as_ref();
    let mut at = 0;
    while at < tokens.len() {
        let token = &tokens[at];

        let is_unordered_item = matches!(token, Token::ListItem(None));
        let is_ordered_item = matches!(token, Token::ListItem(Some(_)));
        let is_line_break = matches!(token, Token::LineBreak);
        let is_text = matches!(token, Token::Text(_) | Token::Url { .. });
        let is_before_eof = at + 1 >= tokens.len();

        // TODO: break this down further
        if !in_unordered_list && is_unordered_item {
            buf.push_str("<ul>\n");
            in_unordered_list = true;
        } else if (was_line_break || !is_line_break)
            && (!is_unordered_item || is_line_break)
            && in_unordered_list
        {
            buf.push_str("</ul>\n");
            in_unordered_list = false;
        }

        // TODO: break this down further
        if !in_ordered_list && is_ordered_item {
            buf.push_str("<ol>\n");
            in_ordered_list = true;
        } else if (was_line_break || !is_line_break)
            && (!is_ordered_item || is_line_break)
            && in_ordered_list
        {
            buf.push_str("</ol>\n");
            in_ordered_list = false;
        }

        if in_paragraph {
            if is_before_eof {
                if !is_line_break {
                    at = write_token_as_html(buf, tokens, at);
                }
                buf.push_str("</p>");
                if is_line_break {
                    at = write_token_as_html(buf, tokens, at);
                }
                in_paragraph = false;
            } else if !is_text
                && is_line_break
                && matches!(tokens.get(at + 1).unwrap(), Token::LineBreak)
            {
                buf.push_str("</p>");
                in_paragraph = false;
            } else {
                at = write_token_as_html(buf, tokens, at);
            }
        } else if is_text {
            buf.push_str("<p>");
            in_paragraph = true;
        } else {
            at = write_token_as_html(buf, tokens, at);
        }

        was_line_break = is_line_break;
    }
}

fn write_text<W: Write>(buf: &mut W, t: &Text) {
    let Text {
        value,
        bold,
        italic,
        code,
    } = t;

    let (bold_s, bold_e) = bold.then(|| ("<b>", "</b>")).unwrap_or_default();
    let (italic_s, italic_e) = italic.then(|| ("<i>", "</i>")).unwrap_or_default();
    let (code_s, code_e) = code.then(|| ("<code>", "</code>")).unwrap_or_default();

    write!(
        buf,
        "{}{}{}{}{}{}{} ",
        code_s, bold_s, italic_s, value, italic_e, bold_e, code_e
    )
    .unwrap()
}

fn write_until_line_break<W: Write>(buf: &mut W, tokens: &[Token], mut at: usize) -> usize {
    while at < tokens.len() {
        if matches!(&tokens[at], Token::LineBreak) {
            break;
        }
        at = write_token_as_html(buf, tokens, at);
    }
    at
}

fn write_token_as_html<W: Write>(buf: &mut W, tokens: &[Token], mut at: usize) -> usize {
    match &tokens[at] {
        Token::Text(t) => write_text(buf, t),
        Token::CodeFence { code, attrs: _ } => {
            write!(buf, "<pre><code>{}</code></pre>", code).unwrap()
        }
        Token::Header(depth) => {
            write!(buf, "<h{}>", depth).unwrap();
            at += 1;
            at = write_until_line_break(buf, tokens, at);
            write!(buf, "</h{}>", depth).unwrap();
            return at;
        }
        Token::Url {
            name,
            url,
            is_image,
        } => {
            if *is_image {
                write!(buf, r#"<img src="{}" alt=""#, url).unwrap();
                if let Some(t) = name {
                    write_text(buf, t);
                } else {
                    buf.write_str(url).unwrap();
                }
                buf.write_str(r#"">"#).unwrap()
            } else {
                write!(buf, r#"<a href="{}">"#, url).unwrap();
                if let Some(t) = name {
                    write_text(buf, t);
                } else {
                    buf.write_str(url).unwrap();
                }
                buf.write_str("</a>").unwrap()
            }
        }
        Token::ListItem(place) => {
            if let Some(place) = place {
                write!(buf, "<li value=\"{}\">", place).unwrap();
            } else {
                buf.write_str("<li>").unwrap();
            }
            at += 1;
            at = write_until_line_break(buf, tokens, at);
            buf.write_str("</li>").unwrap();
            return at;
        }
        Token::LineBreak => buf.write_char('\n').unwrap(),
    }
    at + 1
}
