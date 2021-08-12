use crate::parser::{Text, Token};

use super::*;
use alloc::{format, vec};

#[test]
fn just_text() {
    assert_eq!(
        "asdfdas".parse_md(),
        vec![Text::naked("asdfdas").into_token()]
    )
}

#[test]
fn naked_url() {
    assert_eq!(
        "<asdasd>".parse_md(),
        vec![Token::Url {
            is_image: false,
            name: None,
            url: "asdasd",
        }],
    )
}

#[test]
fn code() {
    assert_eq!(
        "`coding`".parse_md(),
        vec![Text::code("coding").into_token()]
    );
    assert_eq!(
        "`coding``".parse_md(),
        vec![Text::code("coding").into_token()]
    );
    assert_eq!("````".parse_md(), vec![]);
}

#[test]
fn code_fence() {
    assert_eq!(
        "```\ntest```".parse_md(),
        vec![Token::CodeFence {
            attrs: "",
            code: "test",
        }]
    );
    assert_eq!(
        "```rust\ntest```".parse_md(),
        vec![Token::CodeFence {
            attrs: "rust",
            code: "test",
        }]
    );
    assert_eq!(
        "```rust,norun\ntest```".parse_md(),
        vec![Token::CodeFence {
            attrs: "rust,norun",
            code: "test",
        }]
    );
}

#[test]
fn bold_or_italic_text() {
    fn text_test(parsed: Vec<Token<()>>, bold: bool, italic: bool) {
        assert_eq!(
            parsed,
            vec![Token::Text(Text {
                value: "ada",
                bold,
                italic,
                code: false,
            })],
        );
    }

    text_test("ada".parse_md(), false, false);
    text_test("*ada*".parse_md(), false, true);
    text_test("**ada**".parse_md(), true, false);
    text_test("***ada***".parse_md(), true, true);

    assert_eq!(
        "***ada**".parse_md(),
        vec![Token::Text(Text {
            value: "*ada",
            bold: true,
            ..Default::default()
        })],
    );
    assert_eq!(
        "**ada*".parse_md(),
        vec![Token::Text(Text {
            value: "*ada",
            italic: true,
            ..Default::default()
        })],
    );
    assert_eq!(
        "**ada***".parse_md(),
        vec![Token::Text(Text {
            value: "ada",
            bold: true,
            ..Default::default()
        })],
    );
    assert_eq!(
        "*ada**".parse_md(),
        vec![Token::Text(Text {
            value: "ada",
            italic: true,
            ..Default::default()
        })],
    );

    assert_eq!(
        "**g** *g*".parse_md(),
        vec![
            Token::Text(Text {
                value: "g",
                bold: true,
                ..Default::default()
            }),
            Token::Text(Text {
                value: "g",
                italic: true,
                ..Default::default()
            })
        ],
    );
}

#[test]
fn header() {
    const HEADER: &str = "# asdasd";
    const HEADER_3: &str = "### asdasd";
    const NOT_HEADER: &str = "#asdasd";
    const HEADER_AFTER_HEADER: &str = "# #asdf";

    let parsed = HEADER.parse_md();
    assert_eq!(
        parsed,
        vec![Token::Header(1), Text::naked("asdasd").into_token(),],
    );

    let parsed = HEADER_3.parse_md();
    assert_eq!(
        parsed,
        vec![Token::Header(3), Text::naked("asdasd").into_token()],
    );

    let parsed = NOT_HEADER.parse_md();
    assert_eq!(parsed, vec![Text::naked(NOT_HEADER).into_token()]);

    let parsed = HEADER_AFTER_HEADER.parse_md();
    assert_eq!(
        parsed,
        vec![Token::Header(1), Text::naked("#asdf").into_token()],
    );
}

fn ordered_test(parsed: Vec<Token<()>>, place: usize) {
    assert_eq!(
        parsed,
        vec![
            Token::ListItem(Some(place)),
            Text::naked("ada").into_token(),
        ],
    );
}

#[test]
fn ordered_lists() {
    for place in 0..=100 {
        ordered_test(format!("{}. ada", place).parse_md(), place);
    }
}

#[test]
#[should_panic]
fn ordered_list_wrong() {
    for place in 0..=100 {
        ordered_test(format!("{}.ada", place).parse_md(), place);
    }
}

#[test]
fn unordered_lists() {
    fn unordered_test(parsed: Vec<Token<()>>) {
        assert_eq!(
            parsed,
            vec![Token::ListItem(None), Text::naked("ada").into_token()]
        );
    }

    unordered_test("- ada".parse_md());
    unordered_test("+ ada".parse_md());
    unordered_test("* ada".parse_md());
}

#[test]
fn html_paragraph_no_newline() {
    assert_eq!(
        &render_as_html("asdfadsfas".parse_md()),
        "<p>asdfadsfas </p>"
    )
}

#[test]
fn html_paragraph_newline() {
    assert_eq!(
        &render_as_html("asdfadsfas\n".parse_md()),
        "<p>asdfadsfas </p>\n"
    )
}

#[test]
fn html_paragraph_two_newline() {
    assert_eq!(
        &render_as_html("asdfadsfas\n\n".parse_md()),
        "<p>asdfadsfas </p>\n\n"
    )
}

#[test]
fn html_paragraph_newline_paragraph() {
    assert_eq!(
        &render_as_html("asdfadsfas\nasdfasd".parse_md()),
        "<p>asdfadsfas \nasdfasd </p>"
    )
}

#[test]
fn html_paragraph_two_newline_paragraph() {
    assert_eq!(
        &render_as_html("asdfadsfas\n\nasdfas".parse_md()),
        "<p>asdfadsfas </p>\n\n<p>asdfas </p>"
    )
}

#[test]
fn weird_md() {
    const WEIRD_MD: &str = include_str!("../examples/weird.md");
    let output = vec![
        Text::naked("asdf ").into_token(),
        Text::code("asdfasdf").into_token(),
        Token::LineBreak,
        Token::LineBreak,
        Token::CodeFence {
            code:
                "asdfasdf\n\n\n\n# asdfasdf\n\n!!! ** ** *11*   *\n\n\\\\1***13\n\n##!\n\n``\n`\n\n",
            attrs: "",
        },
        Token::LineBreak,
        Token::LineBreak,
        Text::naked("123123").into_token(),
        Token::LineBreak,
    ];
    assert_eq!(WEIRD_MD.parse_md(), output)
}

#[test]
fn text_seperating() {
    assert_eq!(
        "asdfadsf `asdf` <example>".parse_md(),
        vec![
            Text::naked("asdfadsf ").into_token(),
            Text::code("asdf").into_token(),
            Token::Url {
                name: None,
                url: "example",
                is_image: false
            }
        ]
    )
}

const MD: &str = include_str!("../examples/all.md");

#[test]
fn to_html() {
    let html = render_as_html(&MD.parse_md());
    assert_eq!(&html, include_str!("../examples/all.html"));
}

#[test]
fn to_svg() {
    let svg = render_as_svg(&MD.parse_md(), SvgConfig::default());
    assert_eq!(&svg, include_str!("../examples/all.svg"));
}
