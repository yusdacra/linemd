use super::*;
use core::fmt::{self, Display, Formatter, Write};

pub fn render_as_svg(tokens: alloc::vec::Vec<Token>, without_dimensions: bool) -> String {
    let mut doc = String::new();
    let mut text = String::new();
    let mut text_before: usize = 1;
    let mut tspan_before: usize = 0;

    if without_dimensions {
        doc.insert_str(
            0,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.1">"#,
        );
    }

    for token in tokens {
        match token {
            Token::LineBreak => {
                try_apply_text(&mut doc, &mut text, &mut text_before, &mut tspan_before)
            }
            Token::CodeFence { code, attrs: _ } => {
                for line in code.lines() {
                    let span = TSpan::new()
                        .content(line)
                        .font_family("monospace")
                        .x(Position::Absolute(0.0))
                        .y(Position::Relative(1.2));
                    write!(text, "{}", span).unwrap();
                }
            }
            Token::Header {
                depth,
                text: text_token,
            } => {
                let size = match depth {
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
                text_before += 7_usize.saturating_sub(depth) / 4;
                try_apply_text_token(
                    &mut text,
                    *text_token,
                    TSpan::new().font_size(size),
                    &mut tspan_before,
                );
                try_apply_text(&mut doc, &mut text, &mut text_before, &mut tspan_before);
                text_before += 7_usize.saturating_sub(depth) / 4;
            }
            Token::ListItem {
                place,
                text: text_token,
            } => {
                let prefix = if let Some(place) = place {
                    format!("{}. ", place)
                } else {
                    "- ".to_string()
                };
                try_apply_text_token(
                    &mut text,
                    *text_token,
                    TSpan::new().prefix(prefix.as_str()),
                    &mut tspan_before,
                );
            }
            token => try_apply_text_token(&mut text, token, TSpan::new(), &mut tspan_before),
        }
    }

    try_apply_text(&mut doc, &mut text, &mut text_before, &mut tspan_before);

    if !without_dimensions {
        doc.insert_str(
            0,
            &format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="100%" height="{}em" xmlns="http://www.w3.org/2000/svg" version="1.1">"#,
                calculate_content_height(text_before + 1),
            ),
        );
    }
    write!(doc, "</svg>").unwrap();

    doc
}

#[derive(Clone, Copy)]
enum Position {
    Relative(f32),
    Absolute(f32),
}

impl Default for Position {
    fn default() -> Self {
        Position::Relative(0.0)
    }
}

#[derive(Default)]
struct TSpan<'a> {
    prefix: Option<&'a str>,
    content: &'a str,
    font_family: Option<&'a str>,
    font_size: Option<&'a str>,
    font_weight: Option<&'a str>,
    font_style: Option<&'a str>,
    color: Option<&'a str>,
    x: Position,
    y: Position,
}

impl<'a> TSpan<'a> {
    fn new() -> Self {
        Default::default()
    }

    fn prefix(mut self, value: &'a str) -> Self {
        self.prefix = Some(value);
        self
    }

    fn content(mut self, value: &'a str) -> Self {
        self.content = value;
        self
    }

    fn font_family(mut self, value: &'a str) -> Self {
        self.font_family = Some(value);
        self
    }

    fn font_size(mut self, value: &'a str) -> Self {
        self.font_size = Some(value);
        self
    }

    fn font_weight(mut self, value: &'a str) -> Self {
        self.font_weight = Some(value);
        self
    }

    fn font_style(mut self, value: &'a str) -> Self {
        self.font_style = Some(value);
        self
    }

    fn color(mut self, value: &'a str) -> Self {
        self.color = Some(value);
        self
    }

    fn x(mut self, value: Position) -> Self {
        self.x = value;
        self
    }

    fn y(mut self, value: Position) -> Self {
        self.y = value;
        self
    }
}

impl<'a> Display for TSpan<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<tspan")?;
        match self.x {
            Position::Absolute(val) => write!(f, r#" x="{}""#, val)?,
            Position::Relative(val) => write!(f, r#" dx="{}em""#, val)?,
        }
        match self.y {
            Position::Absolute(val) => write!(f, r#" y="{}""#, val)?,
            Position::Relative(val) => write!(f, r#" dy="{}em""#, val)?,
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
        write!(f, ">{}{}</tspan>", self.prefix.unwrap_or(""), self.content)
    }
}

fn calculate_content_height(text_before: usize) -> f32 {
    (text_before * 12) as f32 / 10_f32
}

fn try_apply_text(
    doc: &mut String,
    text: &mut String,
    text_before: &mut usize,
    tspan_before: &mut usize,
) {
    if text.is_empty() {
        *text_before += 1;
    } else {
        let y = calculate_content_height(*text_before);
        write!(doc, r#"<text x="0" y="{}em">{}</text>"#, y, text).unwrap();
        text.clear();
        *text_before += 1;
        *tspan_before = 0;
    }
}

fn try_apply_text_token<'a>(
    text: &mut String,
    token: Token,
    mut span: TSpan<'a>,
    tspan_before: &mut usize,
) {
    span = span.x(Position::Relative(if *tspan_before > 0 {
        0.3
    } else {
        0.0
    }));
    match token {
        Token::Text {
            value,
            bold,
            italic,
        } => {
            if bold {
                span = span.font_weight("bold");
            }
            if italic {
                span = span.font_style("italic");
            }
            write!(text, "{}", span.content(value.trim())).unwrap();
            *tspan_before += 1;
        }
        Token::Code(code) => {
            span = span.font_family("monospace");
            write!(text, "{}", span.content(code.trim())).unwrap();
            *tspan_before += 1;
        }
        Token::Url {
            name,
            is_image: _,
            url,
        } => {
            write!(text, r#"<a xlink:href="{}" target="_blank">"#, url).unwrap();
            let name = name.map_or_else(
                || Token::Text {
                    value: url,
                    bold: false,
                    italic: false,
                },
                |token| *token,
            );
            try_apply_text_token(text, name, span.color("blue"), tspan_before);
            text.push_str("</a>");
        }
        _ => {}
    }
}
