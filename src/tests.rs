use super::*;

#[test]
fn just_text() {
    assert_eq!(
        parse("asdfdas"),
        vec![Token::Text {
            value: "asdfdas".to_string(),
            bold: false,
            italic: false,
        },]
    )
}

#[test]
fn naked_url() {
    assert_eq!(
        parse("<asdasd>"),
        vec![Token::Url {
            is_image: false,
            name: None,
            url: "asdasd".to_string(),
        }],
    )
}

#[test]
fn code() {
    assert_eq!(parse("`coding`"), vec![Token::Code("coding".to_string()),]);
    assert_eq!(
        parse("`coding``"),
        vec![
            Token::Code("coding".to_string()),
            Token::Text {
                value: "`".to_string(),
                bold: false,
                italic: false,
            }
        ]
    );
}

#[test]
fn bold_or_italic_text() {
    fn text_t(v: impl AsRef<str>, bold: bool, italic: bool) {
        let v = v.as_ref();
        assert_eq!(
            parse(v),
            vec![Token::Text {
                value: v.to_string(),
                bold,
                italic,
            },],
        );
    }

    fn text_test(parsed: Vec<Token>, bold: bool, italic: bool) {
        assert_eq!(
            parsed,
            vec![Token::Text {
                value: "ada".to_string(),
                bold,
                italic,
            },],
        );
    }

    text_test(parse("ada"), false, false);
    text_test(parse("*ada*"), false, true);
    text_test(parse("**ada**"), true, false);
    text_test(parse("***ada***"), true, true);

    assert_eq!(
        parse("***ada**"),
        vec![Token::Text {
            value: "*ada".to_string(),
            bold: true,
            italic: false,
        },],
    );
    assert_eq!(
        parse("**ada*"),
        vec![Token::Text {
            value: "*ada".to_string(),
            italic: true,
            bold: false,
        },],
    );
    assert_eq!(
        parse("**ada***"),
        vec![
            Token::Text {
                value: "ada".to_string(),
                bold: true,
                italic: false,
            },
            Token::Text {
                value: "*".to_string(),
                bold: false,
                italic: false,
            }
        ],
    );
    assert_eq!(
        parse("*ada**"),
        vec![
            Token::Text {
                value: "ada".to_string(),
                italic: true,
                bold: false,
            },
            Token::Text {
                value: "*".to_string(),
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
        parse("**g** *g*"),
        vec![
            Token::Text {
                value: "g".to_string(),
                bold: true,
                italic: false,
            },
            Token::Text {
                value: "g".to_string(),
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

    let parsed = parse(HEADER);
    assert_eq!(
        parsed,
        vec![Token::Header {
            depth: 1,
            text: Token::Text {
                value: "asdasd".to_string(),
                italic: false,
                bold: false,
            }
            .into()
        }],
    );

    let parsed = parse(HEADER_3);
    assert_eq!(
        parsed,
        vec![Token::Header {
            depth: 3,
            text: Token::Text {
                value: "asdasd".to_string(),
                italic: false,
                bold: false,
            }
            .into(),
        }],
    );

    let parsed = parse(NOT_HEADER);
    assert_eq!(
        parsed,
        vec![Token::Text {
            value: NOT_HEADER.to_string(),
            bold: false,
            italic: false,
        },],
    );

    let parsed = parse(HEADER_AFTER_HEADER);
    assert_eq!(
        parsed,
        vec![Token::Header {
            depth: 1,
            text: Token::Text {
                value: "#asdf".to_string(),
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
                    value: "ada".to_string(),
                    italic: false,
                    bold: false,
                }
                .into(),
            }]
        );
    }

    for place in 0..=100 {
        ordered_test(parse(format!("{}. ada", place)), place);
    }
    for place in 0..=100 {
        ordered_test(parse(format!("{}.ada", place)), place);
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
                    value: "ada".to_string(),
                    italic: false,
                    bold: false,
                }
                .into(),
            }]
        );
    }

    unordered_test(parse("- ada"));
    unordered_test(parse("+ ada"));
    unordered_test(parse("* ada"));
}
