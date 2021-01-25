#[derive(Debug, Clone, Default)]
pub struct Parser {
    input: String,
    at: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, input: impl AsRef<str>) {
        self.input += input.as_ref();
    }

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

        let token = match self.next_char() {
            '\n' => {
                self.consume_char();
                Token::LineBreak
            }
            '#' if self
                .after_while(|c| is_header(c) && is_not_newline(c))
                .map_or(false, char::is_whitespace) =>
            {
                self.parse_header()
            }
            '+' | '-' | '*' if self.char_at(self.at + 1).map_or(false, char::is_whitespace) => {
                self.parse_unordered_list()
            }
            '0'..='9'
                if self
                    .after_while(|c| c.is_numeric() && is_not_newline(c))
                    .map_or(false, |c| c == '.') =>
            {
                self.parse_ordered_list()
            }
            '<' if self
                .after_while(|c| c != '>' && is_not_newline(c))
                .is_some() =>
            {
                self.parse_naked_url()
            }
            '`' if self.code_end() => self.parse_code(),
            _ => self.parse_text(),
        };

        Some(token)
    }

    fn parse_header(&mut self) -> Token {
        let depth = self.consume_while(is_header).len();
        self.consume_whitespace();
        let text = self.parse_text().into();
        Token::Header { depth, text }
    }

    fn parse_ordered_list(&mut self) -> Token {
        let place = self.consume_while(char::is_numeric).parse().unwrap();
        self.consume_char();
        self.consume_whitespace();
        let text = self.parse_text().into();

        Token::ListItem {
            place: Some(place),
            text,
        }
    }

    fn parse_unordered_list(&mut self) -> Token {
        self.consume_char();
        self.consume_whitespace();
        let text = self.parse_text().into();

        Token::ListItem { place: None, text }
    }

    fn parse_naked_url(&mut self) -> Token {
        self.consume_char();
        let url = self.consume_while(|c| c != '>');
        self.consume_char();

        Token::Url {
            is_image: false,
            name: None,
            url,
        }
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

    fn parse_code(&mut self) -> Token {
        self.consume_char();
        let code = self.consume_while(|c| c != '`');
        self.consume_char();

        Token::Code(code)
    }

    fn parse_text(&mut self) -> Token {
        if let Some((stars, (start, end))) = self.find_star_pair() {
            println!("pair: stars: {}, range: {} - {}", stars, stars, end);
            for _ in 0..stars {
                self.consume_char();
            }
            let mut text = String::with_capacity(end - start);
            for _ in start..end {
                text.push(self.consume_char());
            }
            for _ in 0..stars {
                self.consume_char();
            }
            Token::Text {
                value: text,
                italic: stars != 2,
                bold: stars != 1,
            }
        } else {
            let mut result = String::new();
            loop {
                if !self.eol() {
                    let ch = self.next_char();
                    if is_not_newline(ch)
                        && ch != '<'
                        && (ch != '`' || !self.code_end())
                        && self.find_star_pair().is_none()
                    {
                        result.push(self.consume_char());
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
        }
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

    fn next_char(&self) -> char {
        self.char_at(self.at).unwrap()
    }

    fn char_at(&self, n: usize) -> Option<char> {
        self.input.chars().nth(n)
    }

    fn eol(&self) -> bool {
        self.at >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let ch = self.next_char();
        self.at += 1;
        ch
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, cond: F) -> String {
        let mut result = String::new();
        while !self.eol() && cond(self.next_char()) {
            result.push(self.consume_char());
        }
        result
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Text {
        value: String,
        bold: bool,
        italic: bool,
    },
    Url {
        name: Option<Box<Token>>,
        url: String,
        is_image: bool,
    },
    Header {
        depth: usize,
        text: Box<Token>,
    },
    ListItem {
        /// If `None`, then it is unordered.
        place: Option<usize>,
        text: Box<Token>,
    },
    Code(String),
    LineBreak,
}
