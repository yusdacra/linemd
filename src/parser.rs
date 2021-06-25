use core::{ops::Not, slice::SliceIndex};

use super::*;

#[derive(Debug, Clone, Copy)]
pub enum ParserError {
    EOF,
}

pub type AtWith<T> = (T, usize);
pub type AtStr<'a> = AtWith<&'a str>;
pub type AtToken<'a> = AtWith<Token<'a>>;

pub trait Parser {
    /// Parse the feeded inputs and return tokens.
    fn parse_md(&self) -> Vec<Token<'_>> {
        let mut tokens = Vec::new();
        let mut at = 0;
        while let Some((token, nat)) = self.parse_token(at) {
            at = nat;
            tokens.push(token);
        }
        tokens
    }
    fn parse_token(&self, at: usize) -> Option<AtToken<'_>> {
        self.eof(at)
            .not()
            .then(|| {
                self.consume_whitespace(at)
                    .map(|at| {
                        self.parse_header(at)
                            .or_else(|| self.parse_code(at))
                            .or_else(|| self.parse_text(at))
                    })
                    .flatten()
            })
            .flatten()
    }
    fn parse_code(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, is_backtick)
            .map_or_else(try_handle_err, |v| {
                v.map(|(_, nat)| {
                    let len = nat - at;
                    match len {
                        3 => self
                            .consume_until_str(nat, "```")
                            .map_or_else(try_handle_err, |v| {
                                v.map(|(v, at)| {
                                    let part_count = v.split('\n').count();

                                    let token = if part_count > 1 {
                                        let mut split = v.split('\n');
                                        let attrs_raw = split.next().unwrap();
                                        let attrs = attrs_raw
                                            .split(',')
                                            .flat_map(|s| s.is_empty().not().then(|| s))
                                            .collect();
                                        let code = v.trim_start_matches(attrs_raw);
                                        Token::CodeFence { code, attrs }
                                    } else {
                                        Token::CodeFence {
                                            code: v,
                                            attrs: Vec::new(),
                                        }
                                    };

                                    Some((token, at + 3))
                                })
                                .flatten()
                            }),
                        1 => self
                            .consume_while(nat, |c| is_backtick(c).not())
                            .map_or_else(try_handle_err, |v| {
                                v.map(|(v, at)| (Token::Code(v), at + 1))
                            }),
                        _ => None,
                    }
                })
                .flatten()
            })
    }
    fn parse_header(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, is_header)
            .map_or_else(try_handle_err, |v| {
                v.map(|(_, nat)| {
                    let h = nat - at;
                    if h > 0 {
                        let (content, nnat) = self.parse_token(nat)?;
                        Some((
                            Token::Header {
                                depth: h.max(1).min(6),
                                text: content.into(),
                            },
                            nnat,
                        ))
                    } else {
                        None
                    }
                })
                .flatten()
            })
    }
    fn parse_text(&self, at: usize) -> Option<AtToken<'_>> {
        handle_text_consume(self.consume_while(at, is_not_newline))
    }
    fn consume_whitespace(&self, at: usize) -> Option<usize> {
        self.consume_while(at, char::is_whitespace)
            .map_or_else(
                |(err, maybe_info)| match err {
                    ParserError::EOF => maybe_info.map(|info| info.1),
                },
                |d| d.map(|d| d.1),
            )
            .or(Some(at))
    }
    fn consume_while<F: Fn(char) -> bool>(
        &self,
        mut at: usize,
        f: F,
    ) -> Result<Option<AtStr>, (ParserError, Option<AtStr>)> {
        let old_at = at;
        loop {
            let (ch, nat) = self.consume_char(at).map_err(|err| {
                (err, {
                    let content = self.get_range_str(old_at..at);
                    content.is_empty().not().then(|| (content, at))
                })
            })?;
            if f(ch).not() {
                let content = self.get_range_str(old_at..at);
                return Ok(content.is_empty().not().then(|| (content, at)));
            }
            at = nat;
        }
    }
    fn consume_until_str(
        &self,
        mut at: usize,
        s: &str,
    ) -> Result<Option<AtStr>, (ParserError, Option<AtStr>)> {
        let old_at = at;
        loop {
            let (_, nat) = self.consume_char(at).map_err(|err| {
                (err, {
                    let content = self.get_range_str(old_at..at);
                    content.is_empty().not().then(|| (content, at))
                })
            })?;
            if self.get_range_str(at..).starts_with(s) {
                let content = self.get_range_str(old_at..at);
                return Ok(content.is_empty().not().then(|| (content, at)));
            }
            at = nat;
        }
    }
    #[inline(always)]
    fn eof(&self, at: usize) -> bool {
        self.next_char(at).is_err()
    }
    #[inline(always)]
    fn consume_char(&self, at: usize) -> Result<(char, usize), ParserError> {
        self.next_char(at).map(|c| (c, at + char_bytes(c)))
    }
    fn get_range_str<S: SliceIndex<str>>(&self, range: S) -> &S::Output;
    fn next_char(&self, at: usize) -> Result<char, ParserError>;
}

impl<'a> Parser for &'a str {
    #[inline(always)]
    fn next_char(&self, at: usize) -> Result<char, ParserError> {
        self.chars().nth(at).ok_or(ParserError::EOF)
    }

    #[inline(always)]
    fn get_range_str<S: SliceIndex<str>>(&self, range: S) -> &S::Output {
        &self[range]
    }
}

impl Parser for String {
    #[inline(always)]
    fn next_char(&self, at: usize) -> Result<char, ParserError> {
        self.chars().nth(at).ok_or(ParserError::EOF)
    }

    #[inline(always)]
    fn get_range_str<S: SliceIndex<str>>(&self, range: S) -> &S::Output {
        &self.as_str()[range]
    }
}

#[inline(always)]
fn handle_text_consume<'a>(
    d: Result<Option<AtStr<'a>>, (ParserError, Option<AtStr<'a>>)>,
) -> Option<AtToken<'a>> {
    d.map_or_else(try_handle_err, |v| {
        v.map(|(s, at)| {
            (
                Token::Text {
                    value: s,
                    bold: false,
                    italic: false,
                },
                at,
            )
        })
    })
}

#[inline(always)]
fn try_handle_err(err: (ParserError, Option<AtStr<'_>>)) -> Option<AtToken<'_>> {
    let (err, maybe_info) = err;
    match err {
        ParserError::EOF => maybe_info.map(|(s, at)| {
            (
                Token::Text {
                    value: s,
                    bold: false,
                    italic: false,
                },
                at,
            )
        }),
    }
}

#[inline(always)]
fn char_bytes(c: char) -> usize {
    let mut temp = [0_u8; 4];
    let temp = c.encode_utf8(&mut temp);
    temp.len()
}

#[inline(always)]
const fn is_newline(c: char) -> bool {
    c == '\n'
}

#[inline(always)]
const fn is_not_newline(c: char) -> bool {
    !is_newline(c)
}

#[inline(always)]
const fn is_header(c: char) -> bool {
    c == '#'
}

#[inline(always)]
const fn is_backtick(c: char) -> bool {
    c == '`'
}

/// A token from some parsed text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token<'a> {
    /// Some text.
    Text {
        value: &'a str,
        bold: bool,
        italic: bool,
    },
    /// An URL.
    Url {
        /// Name of this URL (ie. the text in `[]`, if it exists).
        name: Option<Box<Token<'a>>>,
        /// Actual URL. Note that this does not get checked to see if it's a valid URL or not.
        url: &'a str,
        is_image: bool,
    },
    /// A header.
    Header { depth: usize, text: Box<Token<'a>> },
    /// A list item, which can be ordered or unordered.
    ListItem {
        /// If `None`, then it is an unordered item.
        place: Option<usize>,
        text: Box<Token<'a>>,
    },
    /// Some code.
    Code(&'a str),
    /// A code fence. (\`\`\`)
    CodeFence { code: &'a str, attrs: Vec<&'a str> },
    /// A line break.
    LineBreak,
}
