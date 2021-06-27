use core::{ops::Not, slice::SliceIndex};

/// Errors that can occur while parsing.
#[derive(Debug, Clone, Copy)]
pub enum ParserError {
    /// Signals that EOF is reached.
    EOF,
}

/// Convenience type alias that is a tuple of some type and an index.
pub type AtWith<T> = (T, usize);

/// Convenience type alias that is a tuple of a str and an index.
pub type AtStr<'a> = AtWith<&'a str>;

/// Convenience type alias that is a tuple of a token and an index.
pub type AtToken<'a> = AtWith<Token<'a>>;

/// Convenience type alias that is a tuple of a text and an index.
pub type AtText<'a> = AtWith<Text<'a>>;

/// The core of this crate. This trait implements markdown parsing, and several utilities.
///
/// Implementing this trait for your own types is very easy, the onyl required methods are `next_char`
/// and `get_range_str`. You can also provide implementations for other methods if you can include a
/// more optimized way for your types.
pub trait Parser {
    /// Parses self for tokens.
    fn parse_md(&self) -> Vec<Token<'_>> {
        let mut tokens = Vec::new();
        self.parse_md_with_buf(&mut tokens);
        tokens
    }
    /// Parses self for tokens, and outputs to a buffer.
    fn parse_md_with_buf<'a>(&'a self, buf: &mut Vec<Token<'a>>) {
        let mut at = 0;
        while let Some((token, nat)) = self.parse_token(at) {
            at = nat;
            buf.push(token);
        }
    }
    fn parse_token(&self, at: usize) -> Option<AtToken<'_>> {
        self.eof(at)
            .not()
            .then(|| {
                self.consume_whitespace(at)
                    .map(|(_, at)| {
                        self.parse_line_break(at)
                            .or_else(|| self.parse_header(at))
                            .or_else(|| self.parse_list_item(at))
                            .or_else(|| self.parse_texty(at))
                    })
                    .flatten()
            })
            .flatten()
    }
    #[inline(always)]
    fn parse_texty(&self, at: usize) -> Option<AtToken<'_>> {
        self.parse_code(at)
            .or_else(|| self.parse_inline_url(at))
            .or_else(|| self.parse_text(at).map(|(t, at)| (t.into_token(), at)))
    }
    fn parse_code(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, is_backtick)
            .ok()
            .flatten()
            .map(|(ticks, nat)| {
                let len = ticks.len();
                match len {
                    3 => self.parse_code_fence(nat),
                    1 => self.parse_inline_code(nat),
                    _ => None,
                }
            })
            .flatten()
    }
    fn parse_inline_code(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, |c| is_backtick(c).not())
            .ok()
            .flatten()
            .map(|(value, at)| {
                (
                    Token::Text(Text {
                        value,
                        code: true,
                        ..Default::default()
                    }),
                    at + 1,
                )
            })
    }
    fn parse_list_item(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, |c| c.is_numeric() || matches!(c, '-' | '+' | '*'))
            .ok()
            .flatten()
            .map(|(place, nat)| {
                self.next_char(nat)
                    .ok()
                    .map(|c| {
                        let place = place.parse::<usize>().ok();
                        let (place, nat) = place
                            .map(|place| (c == '.').then(|| (Some(place), nat + 1)))
                            .flatten()
                            .unwrap_or((None, nat));
                        self.consume_whitespace(nat)
                            .map(|(w, wnat)| {
                                w.is_empty().not().then(|| (Token::ListItem(place), wnat))
                            })
                            .flatten()
                    })
                    .flatten()
            })
            .flatten()
    }
    fn parse_code_fence(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_until_str(at, "```")
            .ok()
            .flatten()
            .map(|(v, at)| {
                let part_count = v.split('\n').count();

                let token = if part_count >= 1 {
                    let mut split = v.split('\n');
                    let attrs_raw = split.next().unwrap();
                    let code = v.trim_start_matches(attrs_raw).trim_start_matches('\n');
                    Token::CodeFence {
                        code,
                        attrs: attrs_raw,
                    }
                } else {
                    Token::CodeFence {
                        code: v.trim_start_matches('\n'),
                        attrs: "",
                    }
                };

                Some((token, at + 3))
            })
            .flatten()
    }
    fn parse_header(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, |c| c == '#')
            .ok()
            .flatten()
            .map(|(_, hnat)| {
                self.consume_whitespace(hnat)
                    .map(|(w, nat)| {
                        w.is_empty()
                            .not()
                            .then(|| {
                                let h = hnat - at;
                                (h > 0).then(|| (Token::Header(h.max(1).min(6)), nat))
                            })
                            .flatten()
                    })
                    .flatten()
            })
            .flatten()
    }
    fn parse_inline_url(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_while(at, |c| c == '<')
            .ok()
            .flatten()
            .map(|(_, nat)| {
                self.consume_while(nat, |c| c != '>')
                    .ok()
                    .flatten()
                    .map(|(url, nat)| {
                        (
                            Token::Url {
                                name: None,
                                is_image: false,
                                url,
                            },
                            nat + 1,
                        )
                    })
            })
            .flatten()
    }
    fn parse_text(&self, at: usize) -> Option<AtText<'_>> {
        self.consume_while(at, |c| c == '*')
            .ok()
            .flatten()
            .map(|(stars, nat)| {
                let count = stars.len();
                (1..=count)
                    .rev()
                    .flat_map(|search| {
                        let check_italic = count == 2 && search == 1;
                        let offset = check_italic.not().then(|| count - search).unwrap_or(0);
                        self.consume_until_str(nat - offset, &stars[0..search])
                            .ok()
                            .flatten()
                            .map(|(s, nnat)| {
                                (
                                    Text {
                                        value: check_italic
                                            .then(|| self.get_range_str(nat - 1..nnat))
                                            .unwrap_or(s),
                                        bold: search != 1,
                                        italic: search != 2,
                                        code: false,
                                    },
                                    nnat + search,
                                )
                            })
                    })
                    .next()
            })
            .flatten()
            .or_else(|| {
                self.consume_while(at, |c| matches!(c, '\n' | '<' | '`' | '*').not())
                    .map_or_else(try_handle_err, |v| v.map(|(s, nat)| (Text::naked(s), nat)))
            })
    }
    fn parse_line_break(&self, at: usize) -> Option<AtToken<'_>> {
        self.consume_char(at)
            .ok()
            .map(|(c, nat)| (c == '\n').then(|| (Token::LineBreak, nat)))
            .flatten()
    }
    fn consume_whitespace(&self, at: usize) -> Option<AtStr<'_>> {
        self.consume_while(at, |c| c != '\n' && c.is_whitespace())
            .unwrap_or_else(|(err, maybe_info)| match err {
                ParserError::EOF => maybe_info,
            })
            .or(Some(("", at)))
    }
    #[inline(always)]
    fn consume_while<F: Fn(char) -> bool>(
        &self,
        at: usize,
        f: F,
    ) -> Result<Option<AtStr>, (ParserError, Option<AtStr>)> {
        self.consume_until(at, |c, _, _| f(c).not())
    }
    fn consume_until<F: Fn(char, usize, usize) -> bool>(
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
            if f(ch, nat, at) {
                let content = self.get_range_str(old_at..at);
                return Ok(content.is_empty().not().then(|| (content, at)));
            }
            at = nat;
        }
    }
    #[inline(always)]
    fn consume_until_str(
        &self,
        at: usize,
        s: &str,
    ) -> Result<Option<AtStr>, (ParserError, Option<AtStr>)> {
        self.consume_until(at, |_, _, at| self.get_range_str(at..).starts_with(s))
    }
    #[inline(always)]
    fn eof(&self, at: usize) -> bool {
        self.next_char(at).is_err()
    }
    #[inline(always)]
    fn consume_char(&self, at: usize) -> Result<(char, usize), ParserError> {
        self.next_char(at).map(|c| (c, at + char_bytes(c)))
    }
    /// Gets a string slice using the provided range.
    fn get_range_str<S: SliceIndex<str>>(&self, range: S) -> &S::Output;
    /// Gets the character on index `at`.
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
fn try_handle_err(err: (ParserError, Option<AtStr<'_>>)) -> Option<AtText<'_>> {
    let (err, maybe_info) = err;
    match err {
        ParserError::EOF => maybe_info.map(|(s, at)| (Text::naked(s), at)),
    }
}

#[inline(always)]
fn char_bytes(c: char) -> usize {
    let mut temp = [0_u8; 4];
    let temp = c.encode_utf8(&mut temp);
    temp.len()
}

#[inline(always)]
const fn is_backtick(c: char) -> bool {
    c == '`'
}

/// A token from some parsed text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token<'a> {
    /// Some text.
    Text(Text<'a>),
    /// An URL.
    Url {
        /// Name of this URL (ie. the text in `[]`, if it exists).
        name: Option<Text<'a>>,
        /// Actual URL. Note that this does not get checked to see if it's a valid URL or not.
        url: &'a str,
        is_image: bool,
    },
    /// A header.
    Header(usize),
    /// A list item, which can be ordered or unordered.
    /// If `None`, then it is an unordered item.
    ListItem(Option<usize>),
    /// A code fence. (\`\`\`)
    CodeFence { code: &'a str, attrs: &'a str },
    /// A line break.
    LineBreak,
}

/// Some text.
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text<'a> {
    /// The actual underlying string value.
    pub value: &'a str,
    /// `true` if this text is bold.
    pub bold: bool,
    /// `true` if this text is italic.
    pub italic: bool,
    /// `true` if this text is code.
    pub code: bool,
}

impl<'a> Text<'a> {
    /// Create a code text.
    pub const fn code(value: &'a str) -> Self {
        Self {
            value,
            code: true,
            italic: false,
            bold: false,
        }
    }

    /// Create a "naked" text, ie. not italic, bold or code.
    pub const fn naked(value: &'a str) -> Self {
        Self {
            value,
            code: false,
            italic: false,
            bold: false,
        }
    }

    /// Convert this text into a token.
    pub const fn into_token(self) -> Token<'a> {
        Token::Text(self)
    }
}
