use super::*;
use alloc::{format, vec};

#[test]
fn just_text() {
    assert_eq!(
        "asdfdas".parse_md(),
        vec![Token::Text {
            value: "asdfdas",
            bold: false,
            italic: false,
        },]
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
    assert_eq!("`coding`".parse_md(), vec![Token::Code("coding"),]);
    assert_eq!(
        "`coding``".parse_md(),
        vec![
            Token::Code("coding"),
            Token::Text {
                value: "`",
                bold: false,
                italic: false,
            }
        ]
    );
    assert_eq!("````".parse_md(), vec![Token::Code(""), Token::Code("")]);
}

#[test]
fn code_fence() {
    assert_eq!(
        "```\ntest```".parse_md(),
        vec![Token::CodeFence {
            attrs: vec![],
            code: "test",
        }]
    );
    assert_eq!(
        "```rust\ntest```".parse_md(),
        vec![Token::CodeFence {
            attrs: vec!["rust"],
            code: "test",
        }]
    );
    assert_eq!(
        "```rust,norun\ntest```".parse_md(),
        vec![Token::CodeFence {
            attrs: vec!["rust", "norun"],
            code: "test",
        }]
    );
}

#[test]
fn bold_or_italic_text() {
    fn text_t(v: impl AsRef<str>, bold: bool, italic: bool) {
        let v = v.as_ref();
        assert_eq!(
            v.parse_md(),
            vec![Token::Text {
                value: v,
                bold,
                italic,
            },],
        );
    }

    fn text_test(parsed: Vec<Token>, bold: bool, italic: bool) {
        assert_eq!(
            parsed,
            vec![Token::Text {
                value: "ada",
                bold,
                italic,
            },],
        );
    }

    text_test("ada".parse_md(), false, false);
    text_test("*ada*".parse_md(), false, true);
    text_test("**ada**".parse_md(), true, false);
    text_test("***ada***".parse_md(), true, true);

    assert_eq!(
        "***ada**".parse_md(),
        vec![Token::Text {
            value: "*ada",
            bold: true,
            italic: false,
        },],
    );
    assert_eq!(
        "**ada*".parse_md(),
        vec![Token::Text {
            value: "*ada",
            italic: true,
            bold: false,
        },],
    );
    assert_eq!(
        "**ada***".parse_md(),
        vec![
            Token::Text {
                value: "ada",
                bold: true,
                italic: false,
            },
            Token::Text {
                value: "*",
                bold: false,
                italic: false,
            }
        ],
    );
    assert_eq!(
        "*ada**".parse_md(),
        vec![
            Token::Text {
                value: "ada",
                italic: true,
                bold: false,
            },
            Token::Text {
                value: "*",
                bold: false,
                italic: false,
            }
        ],
    );

    for i in 1..12 {
        let t = (0..i).map(|_| '*').collect::<String>();
        text_t(t, false, false);
    }

    assert_eq!(
        "**g** *g*".parse_md(),
        vec![
            Token::Text {
                value: "g",
                bold: true,
                italic: false,
            },
            Token::Text {
                value: "g",
                italic: true,
                bold: false,
            }
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
        vec![Token::Header {
            depth: 1,
            text: Token::Text {
                value: "asdasd",
                italic: false,
                bold: false,
            }
            .into()
        }],
    );

    let parsed = HEADER_3.parse_md();
    assert_eq!(
        parsed,
        vec![Token::Header {
            depth: 3,
            text: Token::Text {
                value: "asdasd",
                italic: false,
                bold: false,
            }
            .into(),
        }],
    );

    let parsed = NOT_HEADER.parse_md();
    assert_eq!(
        parsed,
        vec![Token::Text {
            value: NOT_HEADER,
            bold: false,
            italic: false,
        },],
    );

    let parsed = HEADER_AFTER_HEADER.parse_md();
    assert_eq!(
        parsed,
        vec![Token::Header {
            depth: 1,
            text: Token::Text {
                value: "#asdf",
                italic: false,
                bold: false,
            }
            .into()
        }],
    );
}

#[test]
fn ordered_lists() {
    fn ordered_test(parsed: Vec<Token>, place: usize) {
        assert_eq!(
            parsed,
            vec![Token::ListItem {
                place: Some(place),
                text: Token::Text {
                    value: "ada",
                    italic: false,
                    bold: false,
                }
                .into(),
            }]
        );
    }

    for place in 0..=100 {
        ordered_test(format!("{}. ada", place).parse_md(), place);
    }
    for place in 0..=100 {
        ordered_test(format!("{}.ada", place).parse_md(), place);
    }
}

#[test]
fn unordered_lists() {
    fn unordered_test(parsed: Vec<Token>) {
        assert_eq!(
            parsed,
            vec![Token::ListItem {
                place: None,
                text: Token::Text {
                    value: "ada",
                    italic: false,
                    bold: false,
                }
                .into(),
            }]
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
        "<p>asdfadsfas </p>\n"
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
        "<p>asdfadsfas </p>\n<p>asdfas </p>"
    )
}

#[test]
fn invalid_md() {
    const INVALID_MD: &str = include_str!("../examples/invalid.md");
    assert_eq!(INVALID_MD.parse_md(), vec![])
}

#[test]
fn aaaaaaaaa() {
    assert_eq!("asdfadsf `asdf`".parse_md(), vec![])
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
