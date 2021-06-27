use crate::parser::{Text, Token};

use super::*;
use core::fmt::{self, Display, Formatter, Write};

/// Value that specifies the dimensions of an SVG document.
#[derive(Debug)]
pub enum ViewportDimensions<'a> {
    /// Width and height in pixels (px).
    Integer(u32, u32),
    /// "raw" width and height, put in the resulting SVG as-is.
    Raw(&'a str, &'a str),
    /// Only the width in pixels (px) specified, height calculation is left to linemd.
    OnlyWidth(u32),
    /// Only the "raw" width specified, height calculation is left to linemd.
    OnlyWidthRaw(&'a str),
}

impl<'a> Default for ViewportDimensions<'a> {
    fn default() -> Self {
        Self::OnlyWidthRaw("%100")
    }
}

/// SVG rendering configuration for [`render_as_svg`].
#[derive(Default, Debug)]
pub struct Config<'a> {
    dimensions: ViewportDimensions<'a>,
    font_family: Option<&'a str>,
    font_size: Option<&'a str>,
    font_style: Option<&'a str>,
    font_weight: Option<&'a str>,
}

impl<'a> Config<'a> {
    /// Set the dimensions of the resulting SVG document.
    pub const fn dimensions(mut self, value: ViewportDimensions<'a>) -> Self {
        self.dimensions = value;
        self
    }

    /// Set the font family of the resulting SVG document.
    pub const fn font_family(mut self, value: &'a str) -> Self {
        self.font_family = Some(value);
        self
    }

    /// Set the font size of the resulting SVG document.
    pub const fn font_size(mut self, value: &'a str) -> Self {
        self.font_size = Some(value);
        self
    }

    /// Set the font style of the resulting SVG document.
    pub const fn font_style(mut self, value: &'a str) -> Self {
        self.font_style = Some(value);
        self
    }

    /// Set the font weight of the resulting SVG document.
    pub const fn font_weight(mut self, value: &'a str) -> Self {
        self.font_weight = Some(value);
        self
    }

    fn write_start_tag_to(&self, f: &mut dyn Write, unspecified_height: u32) {
        write!(f, "<svg").unwrap();
        match self.dimensions {
            ViewportDimensions::Integer(width, height) => {
                write!(f, r#" width="{}" height="{}""#, width, height).unwrap();
            }
            ViewportDimensions::Raw(width, height) => {
                write!(f, r#" width="{}" height="{}""#, width, height).unwrap();
            }
            ViewportDimensions::OnlyWidth(width) => {
                write!(f, r#" width="{}" height="{}""#, width, unspecified_height).unwrap();
            }
            ViewportDimensions::OnlyWidthRaw(width) => {
                write!(f, r#" width="{}" height="{}""#, width, unspecified_height).unwrap();
            }
        }
        if let Some(value) = self.font_family {
            write!(f, r#" font-family="{}""#, value).unwrap();
        }
        if let Some(value) = self.font_size {
            write!(f, r#" font-size="{}""#, value).unwrap();
        }
        if let Some(value) = self.font_style {
            write!(f, r#" font-style="{}""#, value).unwrap();
        }
        if let Some(value) = self.font_weight {
            write!(f, r#" font-weight="{}""#, value).unwrap();
        }
        write!(f, r#" xmlns="http://www.w3.org/2000/svg" version="1.1">"#).unwrap();
    }

    fn write_end_tag_to(&self, f: &mut dyn Write) {
        write!(f, "</svg>").unwrap();
    }
}

/// Renders parsed tokens as SVG.
///
/// # Example
/// ```
/// # use linemd::{render_as_svg, SvgConfig, Parser};
/// let svg = render_as_svg("Some uninspiring text.".parse_md(), SvgConfig::default());
/// ```
pub fn render_as_svg<'a>(tokens: impl AsRef<[Token<'a>]> + 'a, config: Config<'_>) -> String {
    let mut doc = String::new();
    render_to_buffer(tokens, config, &mut doc);
    doc
}

/// Renders parsed tokens as SVG, to a buffer.
///
/// # Example
/// ```
/// # use linemd::{svg, SvgConfig, Parser};
/// let mut buffer = String::new();
/// let svg = svg::render_to_buffer("Some uninspiring text.".parse_md(), SvgConfig::default(), &mut buffer);
/// ```
pub fn render_to_buffer<'a>(
    tokens: impl AsRef<[Token<'a>]> + 'a,
    config: Config<'_>,
    doc: &mut String,
) {
    let mut text = String::new();
    let mut text_before: u32 = 1;
    let mut tspan_before: u32 = 0;

    let mut at = 0;
    let tokens = tokens.as_ref();
    let mut was_header = None;

    while at < tokens.len() {
        let token = &tokens[at];
        match token {
            Token::LineBreak => {
                try_apply_text(doc, &mut text, &mut text_before, &mut tspan_before);
                if let Some(depth) = was_header {
                    text_before += 7_u32.saturating_sub(depth as u32) / 4;
                }
            }
            Token::CodeFence { code, attrs: _ } => {
                for line in code.lines() {
                    let span = TSpan::<0>::new()
                        .content(line)
                        .font_family("monospace")
                        .x(Position::Absolute(0))
                        .y(Position::Relative(19));
                    write!(text, "{}", span).unwrap();
                }
            }
            Token::Header(depth) => {
                let size = match *depth {
                    1 => "xx-large",
                    2 => "x-large",
                    3 => "large",
                    4 => "small",
                    5 => "x-small",
                    6 => "xx-small",
                    x if x > 6 => "xx-small",
                    x if x < 1 => "xx-large",
                    _ => unreachable!(),
                };
                text_before += 7_u32.saturating_sub(*depth as u32) / 4;
                at += 1;
                at = write_until_line_break(
                    &mut text,
                    TSpan::<0>::new().font_size(size),
                    &mut tspan_before,
                    at,
                    tokens,
                );
                was_header = Some(*depth);
                continue;
            }
            Token::ListItem(place) => {
                at += 1;
                if at >= tokens.len() {
                    continue;
                }
                if let Some(place) = place {
                    let prefix = [Value::Number(*place), Value::Str(". ")];
                    try_apply_text_token(
                        &mut text,
                        &tokens[at],
                        TSpan::<2>::new().prefix(prefix),
                        &mut tspan_before,
                    );
                } else {
                    let prefix = [Value::Str("• ")];
                    try_apply_text_token(
                        &mut text,
                        &tokens[at],
                        TSpan::<1>::new().prefix(prefix),
                        &mut tspan_before,
                    );
                }
                at = write_until_line_break(
                    &mut text,
                    TSpan::<0>::new(),
                    &mut tspan_before,
                    at,
                    tokens,
                );
                continue;
            }
            token => try_apply_text_token(&mut text, &token, TSpan::<0>::new(), &mut tspan_before),
        }
        at += 1;
        was_header = None;
    }

    try_apply_text(doc, &mut text, &mut text_before, &mut tspan_before);

    let content_height = calculate_content_height(text_before + 1);
    let mut tmp = String::new();
    config.write_start_tag_to(&mut tmp, content_height);
    doc.insert_str(0, &tmp);
    config.write_end_tag_to(doc);
}

fn write_until_line_break<'a, const N: usize>(
    text: &mut String,
    span: TSpan<'a, N>,
    tspan_before: &mut u32,
    mut at: usize,
    tokens: &[Token],
) -> usize {
    while at < tokens.len() {
        let token = &tokens[at];
        if matches!(token, Token::LineBreak) {
            break;
        }
        try_apply_text_token(text, &token, span.clone(), tspan_before);
        at += 1;
    }
    at
}

#[derive(Clone)]
enum Value<'a> {
    Number(usize),
    Str(&'a str),
}

#[derive(Clone, Copy)]
enum Position {
    Relative(usize),
    Absolute(usize),
}

impl Default for Position {
    fn default() -> Self {
        Position::Relative(0)
    }
}

#[derive(Clone)]
struct TSpan<'a, const N: usize> {
    prefix: [Value<'a>; N],
    content: &'a str,
    font_family: Option<&'a str>,
    font_size: Option<&'a str>,
    font_weight: Option<&'a str>,
    font_style: Option<&'a str>,
    color: Option<&'a str>,
    x: Position,
    y: Position,
}

impl<'a> Default for TSpan<'a, 0> {
    fn default() -> Self {
        Self {
            prefix: [],
            content: "",
            font_family: None,
            font_size: None,
            font_weight: None,
            font_style: None,
            color: None,
            x: Position::default(),
            y: Position::default(),
        }
    }
}

impl<'a, const N: usize> TSpan<'a, N> {
    fn new() -> TSpan<'a, 0> {
        Default::default()
    }

    fn prefix<const S: usize>(self, value: [Value<'a>; S]) -> TSpan<'a, S> {
        TSpan::<'a> {
            prefix: value,
            content: self.content,
            font_family: self.font_family,
            font_size: self.font_size,
            font_weight: self.font_weight,
            font_style: self.font_style,
            color: self.color,
            x: self.x,
            y: self.y,
        }
    }

    const fn content(mut self, value: &'a str) -> Self {
        self.content = value;
        self
    }

    const fn font_family(mut self, value: &'a str) -> Self {
        self.font_family = Some(value);
        self
    }

    const fn font_size(mut self, value: &'a str) -> Self {
        self.font_size = Some(value);
        self
    }

    const fn font_weight(mut self, value: &'a str) -> Self {
        self.font_weight = Some(value);
        self
    }

    const fn font_style(mut self, value: &'a str) -> Self {
        self.font_style = Some(value);
        self
    }

    const fn color(mut self, value: &'a str) -> Self {
        self.color = Some(value);
        self
    }

    const fn x(mut self, value: Position) -> Self {
        self.x = value;
        self
    }

    const fn y(mut self, value: Position) -> Self {
        self.y = value;
        self
    }
}

impl<'a, const N: usize> Display for TSpan<'a, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<tspan")?;
        match self.x {
            Position::Absolute(val) => write!(f, r#" x="{}""#, val)?,
            Position::Relative(val) => write!(f, r#" dx="{}""#, val)?,
        }
        match self.y {
            Position::Absolute(val) => write!(f, r#" y="{}""#, val)?,
            Position::Relative(val) => write!(f, r#" dy="{}""#, val)?,
        }
        if let Some(value) = self.font_family {
            write!(f, r#" font-family="{}""#, value)?;
        }
        if let Some(value) = self.font_size {
            write!(f, r#" font-size="{}""#, value)?;
        }
        if let Some(value) = self.font_style {
            write!(f, r#" font-style="{}""#, value)?;
        }
        if let Some(value) = self.font_weight {
            write!(f, r#" font-weight="{}""#, value)?;
        }
        if let Some(color) = self.color {
            write!(f, r#" fill="{}""#, color)?;
        }
        f.write_char('>')?;
        for v in &self.prefix {
            match v {
                Value::Number(n) => write!(f, "{}", n)?,
                Value::Str(s) => f.write_str(s)?,
            }
        }
        write!(f, "{}</tspan>", self.content)
    }
}

const fn calculate_content_height(text_before: u32) -> u32 {
    (text_before * 12 * 16) / 10
}

fn try_apply_text(
    doc: &mut String,
    text: &mut String,
    text_before: &mut u32,
    tspan_before: &mut u32,
) {
    if text.is_empty() {
        *text_before += 1;
    } else {
        let y = calculate_content_height(*text_before);
        write!(doc, r#"<text x="0" y="{}">{}</text>"#, y, text).unwrap();
        text.clear();
        *text_before += 1;
        *tspan_before = 0;
    }
}

fn try_apply_text_token<'a, const N: usize>(
    text: &mut String,
    token: &Token,
    mut span: TSpan<'a, N>,
    tspan_before: &mut u32,
) {
    span = span.x(Position::Relative(if *tspan_before > 0 { 5 } else { 0 }));
    match token {
        Token::Text(Text {
            value,
            bold,
            italic,
            code,
        }) => {
            if *bold {
                span = span.font_weight("bold");
            }
            if *italic {
                span = span.font_style("italic");
            }
            if *code {
                span = span.font_family("monospace");
            }
            write!(text, "{}", span.content(value.trim())).unwrap();
            *tspan_before += 1;
        }
        Token::Url {
            name,
            is_image: _,
            url,
        } => {
            write!(text, r#"<a xlink:href="{}" target="_blank">"#, url).unwrap();
            let name = name.as_ref().map_or_else(
                || Token::Text(Text::naked(url)),
                |token| Token::Text(token.clone()),
            );
            try_apply_text_token(text, &name, span.color("blue"), tspan_before);
            text.push_str("</a>");
        }
        _ => {}
    }
}
