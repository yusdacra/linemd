pub fn parse(md: impl AsRef<str>) -> Vec<Token> {
    {
        let mut parser = Parser::new();
        parser.feed(md);
        parser
    }
    .parse()
}

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
        } else if is_newline(self.next_char()) {
            self.consume_char();
            return Some(Token::LineBreak);
        }

        let token = match self.next_char() {
            '#' => {
                if self
                    .after_while(is_header)
                    .map_or(false, |(c, _)| c.is_whitespace())
                {
                    self.parse_header()
                } else {
                    self.parse_text().into()
                }
            }
            '+' | '-' | '*' | '0'..='9' => {
                if self.char_after(1).map_or(false, char::is_whitespace)
                    || self
                        .after_while(char::is_numeric)
                        .map_or(false, |(c, _)| c == '.')
                {
                    self.parse_list()
                } else {
                    self.parse_text().into()
                }
            }
            _ => self.parse_text().into(),
        };

        Some(token)
    }

    fn parse_header(&mut self) -> Token {
        let depth = self.consume_while(is_header).len();
        self.consume_whitespace();
        let text = self.parse_text();
        Token::Header { depth, text }
    }

    fn parse_list(&mut self) -> Token {
        let symbol = self.consume_char();
        let place = match symbol {
            '0'..='9' => {
                let mut place = self.consume_while(char::is_numeric);
                place.insert(0, symbol);
                Some(place.parse().unwrap())
            }
            '-' | '+' | '*' => None,
            _ => unreachable!(),
        };
        if place.is_some() {
            self.consume_char();
        }
        self.consume_whitespace();
        let text = self.parse_text();
        Token::ListItem { place, text }
    }

    fn parse_text(&mut self) -> Text {
        Text {
            value: self.consume_while(is_not_newline),
            ..Default::default()
        }
    }

    fn after_while<F: Fn(char) -> bool>(&self, cond: F) -> Option<(char, usize)> {
        let mut at = self.at;
        while let Some(c) = self.char_after(at) {
            if cond(c) {
                at += 1;
            } else {
                return Some((c, at));
            }
        }
        None
    }

    fn next_char(&self) -> char {
        self.char_after(0).unwrap()
    }

    fn char_after(&self, n: usize) -> Option<char> {
        self.input[self.at..].chars().nth(n)
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
    Text(Text),
    Url {
        name: Option<Text>,
        url: String,
        is_image: bool,
    },
    Header {
        depth: usize,
        text: Text,
    },
    ListItem {
        /// If `None`, then it is unordered.
        place: Option<usize>,
        text: Text,
    },
    Code(String),
    LineBreak,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Text {
    pub value: String,
    pub bold: bool,
    pub italic: bool,
}

impl Into<Token> for Text {
    fn into(self) -> Token {
        Token::Text(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text() {
        fn text_test(parsed: Vec<Token>, bold: bool, italic: bool) {
            assert_eq!(
                parsed,
                vec![Token::Text(Text {
                    value: "ada".to_string(),
                    bold,
                    italic,
                }),],
            );
        }

        text_test(parse("ada"), false, false);
        text_test(parse("*ada*"), false, true);
        text_test(parse("**ada**"), true, false);
        text_test(parse("***ada***"), true, true);

        assert_eq!(
            parse("***ada**"),
            vec![Token::Text(Text {
                value: "***ada**".to_string(),
                ..Default::default()
            }),],
        );
    }

    #[test]
    fn header() {
        const HEADER: &str = "# asdasd";
        const HEADER_3: &str = "### asdasd";
        const NOT_HEADER: &str = "#asdasd";
        const HEADER_AFTER_HEADER: &str = "# #asdf";

        let parsed = parse(HEADER);
        assert_eq!(
            parsed,
            vec![Token::Header {
                depth: 1,
                text: Text {
                    value: "asdasd".to_string(),
                    ..Default::default()
                }
            }],
        );

        let parsed = parse(HEADER_3);
        assert_eq!(
            parsed,
            vec![Token::Header {
                depth: 3,
                text: Text {
                    value: "asdasd".to_string(),
                    ..Default::default()
                },
            }],
        );

        let parsed = parse(NOT_HEADER);
        assert_eq!(
            parsed,
            vec![Token::Text(Text {
                value: NOT_HEADER.to_string(),
                ..Default::default()
            }),],
        );

        let parsed = parse(HEADER_AFTER_HEADER);
        assert_eq!(
            parsed,
            vec![Token::Header {
                depth: 1,
                text: Text {
                    value: "#asdf".to_string(),
                    ..Default::default()
                }
            }],
        );
    }

    #[test]
    fn lists() {
        fn unordered_test(parsed: Vec<Token>) {
            assert_eq!(
                parsed,
                vec![Token::ListItem {
                    place: None,
                    text: Text {
                        value: "ada".to_string(),
                        ..Default::default()
                    },
                }]
            );
        }

        fn ordered_test(parsed: Vec<Token>, place: usize) {
            assert_eq!(
                parsed,
                vec![Token::ListItem {
                    place: Some(place),
                    text: Text {
                        value: "ada".to_string(),
                        ..Default::default()
                    },
                }]
            );
        }

        unordered_test(parse("- ada"));
        unordered_test(parse("+ ada"));
        unordered_test(parse("* ada"));

        for place in 0..=9 {
            ordered_test(parse(format!("{}. ada", place)), place);
        }
        ordered_test(parse("12. ada"), 12);
        ordered_test(parse("12.ada"), 12);
    }
}
