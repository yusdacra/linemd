use super::*;

/// Renders parsed tokens (`Vec<Token>`) as HTML.
///
/// # Example
/// ```
/// # use linemd::{parse, render_as_html};
/// let html = render_as_html(parse("Some uninspiring text."));
/// ```
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

        let tok_html = token_to_html(token);
        html.push_str(&tok_html);

        was_line_break = is_line_break;
    }

    html.trim().to_string()
}

fn surrond_in_html_tag(tag: &str, data: &str) -> String {
    format!("<{}>{}</{}>", tag, data, tag)
}

fn token_to_html(token: Token) -> String {
    match token {
        Token::Text {
            mut value,
            bold,
            italic,
        } => {
            if bold {
                value = surrond_in_html_tag("b", value.as_str());
            }
            if italic {
                value = surrond_in_html_tag("i", value.as_str());
            }
            value.push(' ');
            value
        }
        Token::Code(code) => {
            let mut code = surrond_in_html_tag("code", code.as_str());
            code.push(' ');
            code
        }
        Token::CodeFence { code, attrs: _ } => {
            format!("<code>\n{}\n</code>\n", code)
        }
        Token::Header { depth, text } => {
            let depth = depth.min(6).max(1);
            surrond_in_html_tag(&format!("h{}", depth), token_to_html(*text).as_str())
        }
        Token::Url {
            name,
            url,
            is_image,
        } => {
            if is_image {
                format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    url,
                    name.map_or(url.clone(), |t| token_to_html(*t)),
                )
            } else {
                format!(
                    "<a href=\"{}\">{}</a>",
                    url,
                    name.map_or(url.clone(), |t| token_to_html(*t)),
                )
            }
        }
        Token::ListItem { place, text } => {
            if let Some(place) = place {
                format!("<li value=\"{}\">{}</li>", place, token_to_html(*text))
            } else {
                surrond_in_html_tag("li", token_to_html(*text).as_str())
            }
        }
        Token::LineBreak => "\n".to_string(),
    }
}
