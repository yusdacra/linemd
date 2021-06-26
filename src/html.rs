use super::*;
use core::fmt::{self, Write};

/// Renders parsed tokens (`Vec<Token>`) as HTML.
///
/// # Example
/// ```
/// # use linemd::{render_as_html, Parser};
/// let html = render_as_html("Some uninspiring text.".parse_md());
/// ```
pub fn render_as_html<'a>(tokens: impl AsRef<[Token<'a>]> + 'a) -> String {
    let mut html = String::new();

    let mut unordered_list = false;
    let mut ordered_list = false;

    let mut was_line_break = false;
    let mut paragraph = false;

    let tokens = tokens.as_ref();
    for (index, token) in tokens.iter().enumerate() {
        let is_unordered_item = matches!(token, Token::ListItem { place: None, .. });
        let ordered_item = matches!(token, Token::ListItem { place: Some(_), .. });
        let is_line_break = matches!(token, Token::LineBreak);
        let is_text = matches!(
            token,
            Token::Text { .. } | Token::Code(_) | Token::Url { .. }
        );
        let is_eof = index + 1 >= tokens.len();

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

        if is_text && !paragraph {
            html.push_str("<p>");
            paragraph = true;
        }

        if paragraph {
            if is_eof {
                if !is_line_break {
                    write_token_as_html(&mut html, token).unwrap();
                }
                html.push_str("</p>");
                if is_line_break {
                    write_token_as_html(&mut html, token).unwrap();
                }
                paragraph = false;
            } else if !is_text
                && is_line_break
                && matches!(tokens.get(index + 1).unwrap(), Token::LineBreak)
            {
                html.push_str("</p>");
                paragraph = false;
            } else {
                write_token_as_html(&mut html, token).unwrap();
            }
        } else {
            write_token_as_html(&mut html, token).unwrap();
        }

        was_line_break = is_line_break;
    }

    html
}

fn write_surround_tag<W: Write>(buf: &mut W, tag: &str, data: &str) {
    write!(buf, "<{}>{}</{}>", tag, data, tag).unwrap()
}

fn write_token_as_html<W: Write>(buf: &mut W, token: &Token) -> fmt::Result {
    match token {
        Token::Text {
            value,
            bold,
            italic,
        } => {
            if *bold {
                buf.write_str("<b>")?;
            }
            if *italic {
                buf.write_str("<i>")?;
            }
            buf.write_str(value)?;
            if *italic {
                buf.write_str("</i>")?;
            }
            if *bold {
                buf.write_str("</b>")?;
            }
            buf.write_char(' ')
        }
        Token::Code(code) => {
            write_surround_tag(buf, "code", code);
            buf.write_char(' ')
        }
        Token::CodeFence { code, attrs: _ } => {
            write!(buf, "<pre><code>{}</code></pre>", code)
        }
        Token::Header { depth, text } => {
            let depth = (*depth).min(6).max(1);
            write!(buf, "<h{}>", depth)?;
            write_token_as_html(buf, &text)?;
            write!(buf, "</h{}>", depth)
        }
        Token::Url {
            name,
            url,
            is_image,
        } => {
            if *is_image {
                write!(buf, r#"<img src="{}" alt=""#, url)?;
                if let Some(t) = name {
                    write_token_as_html(buf, &t)?;
                } else {
                    buf.write_str(url)?;
                }
                buf.write_str(r#"">"#)
            } else {
                write!(buf, r#"<a href="{}">"#, url)?;
                if let Some(t) = name {
                    write_token_as_html(buf, &t)?;
                } else {
                    buf.write_str(url)?;
                }
                buf.write_str("</a>")
            }
        }
        Token::ListItem { place, text } => {
            if let Some(place) = place {
                write!(buf, "<li value=\"{}\">", place)?;
                write_token_as_html(buf, &text)?;
            } else {
                buf.write_str("<li>")?;
                write_token_as_html(buf, &text)?;
            }
            buf.write_str("</li>")
        }
        Token::LineBreak => buf.write_char('\n'),
    }
}
