use super::*;

/// A markdown parser.
#[derive(Debug, Clone, Default)]
pub struct Parser {
    input: String,
    at: usize,
}

impl Parser {
    /// Create a new and empty parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add the parser some input, which will be parsed on the next [`Parser::parse()`] call.
    pub fn feed(&mut self, input: &str) {
        self.input += input;
    }

    /// Parse the feeded inputs and return tokens.
    pub fn parse(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(token) = self.parse_token() {
            tokens.push(token);
        }

        tokens
    }

    fn parse_token(&mut self) -> Option<Token> {
        self.consume_whitespace();

        if self.eol() {
            return None;
        }

        let c = self.next_char()?;
        let token = match c {
            '\n' => {
                self.consume_char();
                Token::LineBreak
            }
            '#' if self
                .after_while(|c| is_header(c) && is_not_newline(c))
                .map_or(false, char::is_whitespace) =>
            {
                self.parse_header()?
            }
            '+' | '-' | '*' if self.char_at(self.at + 1).map_or(false, char::is_whitespace) => {
                self.parse_unordered_list()?
            }
            '0'..='9'
                if self
                    .after_while(|c| c.is_numeric() && is_not_newline(c))
                    .map_or(false, |c| c == '.') =>
            {
                self.parse_ordered_list()?
            }
            '<' if self
                .after_while(|c| c != '>' && is_not_newline(c))
                .is_some() =>
            {
                self.parse_naked_url()?
            }
            '`' if self.code_fence() => self.parse_code_fence()?,
            '`' if self.code_end() => self.parse_code()?,
            _ => self.parse_text()?,
        };

        Some(token)
    }

    fn parse_header(&mut self) -> Option<Token> {
        let depth = self.consume_while(is_header)?.len();
        self.consume_whitespace();
        let text = self.parse_text()?.into();
        Some(Token::Header { depth, text })
    }

    fn parse_ordered_list(&mut self) -> Option<Token> {
        let place = self.consume_while(char::is_numeric)?.parse().ok()?;
        self.consume_char();
        self.consume_whitespace();
        let text = self.parse_text()?.into();

        Some(Token::ListItem {
            place: Some(place),
            text,
        })
    }

    fn parse_unordered_list(&mut self) -> Option<Token> {
        self.consume_char();
        self.consume_whitespace();
        let text = self.parse_text()?.into();

        Some(Token::ListItem { place: None, text })
    }

    fn parse_naked_url(&mut self) -> Option<Token> {
        self.consume_char();
        let url = self.consume_while(|c| c != '>')?;
        self.consume_char();

        Some(Token::Url {
            is_image: false,
            name: None,
            url,
        })
    }

    fn code_end(&self) -> bool {
        let mut at = self.at + 1;
        while let Some(c) = self.char_at(at) {
            if c != '`' && is_not_newline(c) {
                at += 1;
            } else {
                return true;
            }
        }
        false
    }

    fn code_fence(&self) -> bool {
        let mut at = self.at;

        let mut ticks_start = 0;

        while let Some(c) = self.char_at(at) {
            if c == '`' {
                ticks_start += 1;
                at += 1;
            } else {
                break;
            }
        }

        if ticks_start != 3 {
            return false;
        }

        while let Some(c) = self.char_at(at) {
            if c != '`' {
                at += 1;
            } else {
                break;
            }
        }

        let mut ticks_end = 0;

        while let Some(c) = self.char_at(at) {
            if c == '`' {
                ticks_end += 1;
                at += 1;
            }
        }

        if ticks_end != 3 {
            return false;
        }

        true
    }

    fn parse_code(&mut self) -> Option<Token> {
        self.consume_char();
        let code = self.consume_while(|c| c != '`')?;
        self.consume_char();

        Some(Token::Code(code))
    }

    fn parse_code_fence(&mut self) -> Option<Token> {
        self.consume_times(3);

        let attrs: Vec<String> = self
            .consume_while(is_not_newline)?
            .split(',')
            .filter_map(|c| {
                if c.is_empty() {
                    None
                } else {
                    Some(c.to_string())
                }
            })
            .collect();

        let code = self
            .consume_while(|c| c != '`')?
            .trim_matches(is_newline)
            .to_string();

        self.consume_times(3);
        Some(Token::CodeFence { attrs, code })
    }

    fn parse_text(&mut self) -> Option<Token> {
        Some(if let Some((stars, (start, end))) = self.find_star_pair() {
            self.consume_times(stars);
            let mut text = String::with_capacity(end - start);
            for _ in start..end {
                text.push(self.consume_char()?);
            }
            self.consume_times(stars);
            Token::Text {
                value: text,
                italic: stars != 2,
                bold: stars != 1,
            }
        } else {
            let mut result = String::new();
            loop {
                if !self.eol() {
                    let ch = self.next_char()?;
                    if is_not_newline(ch)
                        && ch != '<'
                        && (ch != '`' || !self.code_end())
                        && self.find_star_pair().is_none()
                    {
                        result.push(self.consume_char()?);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            Token::Text {
                value: result,
                bold: false,
                italic: false,
            }
        })
    }

    fn find_star_pair(&self) -> Option<(usize, (usize, usize))> {
        let mut at = self.at;

        let range_start;
        let range_end;

        let mut stars_start = 0;

        while let Some(c) = self.char_at(at) {
            if is_not_newline(c) {
                if c == '*' {
                    stars_start += 1;
                } else {
                    break;
                }
                at += 1;
            } else {
                return None;
            }
        }

        if stars_start == 0 {
            return None;
        }

        range_start = at;
        while let Some(c) = self.char_at(at) {
            if is_not_newline(c) {
                if c == '*' {
                    break;
                }
                at += 1;
            } else {
                return None;
            }
        }
        range_end = at;

        let mut stars_end = 0;

        while let Some(c) = self.char_at(at) {
            if is_not_newline(c) {
                if c == '*' {
                    stars_end += 1;
                } else {
                    break;
                }
                at += 1;
            }
        }

        if stars_end == 0 {
            return None;
        }

        let stars: usize = stars_end.min(stars_start);

        if stars > 3 {
            None
        } else {
            let start_cut: usize = stars_start.saturating_sub(stars);
            let end_cut: usize = stars_end.saturating_sub(stars);

            Some((
                stars,
                (range_start + end_cut, range_end + start_cut + end_cut),
            ))
        }
    }

    fn after_while<F: Fn(char) -> bool>(&self, cond: F) -> Option<char> {
        let mut at = self.at;
        while let Some(c) = self.char_at(at) {
            if cond(c) {
                at += 1;
            } else {
                return Some(c);
            }
        }
        None
    }

    fn next_char(&self) -> Option<char> {
        self.char_at(self.at)
    }

    fn char_at(&self, n: usize) -> Option<char> {
        self.input.chars().nth(n)
    }

    fn eol(&self) -> bool {
        self.at >= self.input.len()
    }

    fn consume_char(&mut self) -> Option<char> {
        let ch = self.next_char();
        self.at += 1;
        ch
    }

    fn consume_times(&mut self, times: usize) -> Option<String> {
        let mut result = String::new();
        for _ in 0..times {
            result.push(self.consume_char()?);
        }
        Some(result)
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, cond: F) -> Option<String> {
        let mut result = String::new();
        while !self.eol() && cond(self.next_char()?) {
            result.push(self.consume_char()?);
        }
        Some(result)
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| is_not_newline(c) && c.is_whitespace());
    }
}

fn is_newline(c: char) -> bool {
    c == '\n'
}

fn is_not_newline(c: char) -> bool {
    !is_newline(c)
}

fn is_header(c: char) -> bool {
    c == '#'
}

/// A token from some parsed text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    /// Some text.
    Text {
        value: String,
        bold: bool,
        italic: bool,
    },
    /// An URL.
    Url {
        /// Name of this URL (ie. the text in `[]`, if it exists).
        name: Option<Box<Token>>,
        /// Actual URL. Note that this does not get checkec to see if it's a valid URL or not.
        url: String,
        is_image: bool,
    },
    /// A header.
    Header { depth: usize, text: Box<Token> },
    /// A list item, which can be ordered or unordered.
    ListItem {
        /// If `None`, then it is an unordered item.
        place: Option<usize>,
        text: Box<Token>,
    },
    /// Some code.
    Code(String),
    /// A code fence. (\`\`\`)
    CodeFence { code: String, attrs: Vec<String> },
    /// A line break.
    LineBreak,
}

impl Token {
    /// Consumes this token and returns respective HTML for it.
    pub fn into_html(self) -> String {
        match self {
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
                surrond_in_html_tag(&format!("h{}", depth), text.into_html().as_str())
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
                        name.map_or(url.clone(), |t| t.into_html()),
                    )
                } else {
                    format!(
                        "<a href=\"{}\">{}</a>",
                        url,
                        name.map_or(url.clone(), |t| t.into_html()),
                    )
                }
            }
            Token::ListItem { place, text } => {
                if let Some(place) = place {
                    format!("<li value=\"{}\">{}</li>", place, text.into_html())
                } else {
                    surrond_in_html_tag("li", text.into_html().as_str())
                }
            }
            Token::LineBreak => "\n".to_string(),
        }
    }
}

fn surrond_in_html_tag(tag: &str, data: &str) -> String {
    format!("<{}>{}</{}>", tag, data, tag)
}
