use linemd::{render_as_html, render_as_svg, Parser, SvgConfig};
use std::io::{prelude::*, stdin};

const HELP_TEXT: &str = include_str!("help.txt");

fn main() {
    let help = std::env::args().any(|s| matches!(s.as_str(), "-h" | "--help"));
    let read_stdin = std::env::args().last().map_or(false, |s| s == "-");
    let svg = std::env::args().any(|s| matches!(s.as_str(), "-S" | "--svg"));

    if std::env::args().len() > 1 {
        let md = if help {
            println!("{}", HELP_TEXT);
            return;
        } else if read_stdin {
            let mut input = String::new();
            if let Err(err) = stdin().read_to_string(&mut input) {
                eprintln!("failed to read from stdin: {}", err);
                std::process::exit(2);
            }
            input
        } else {
            let arg = std::env::args().last().unwrap();
            match std::fs::read_to_string(&arg) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("failed to read file '{}': {}", arg, err);
                    std::process::exit(1);
                }
            }
        };

        let tokens = md.parse_md();
        let out = if svg {
            render_as_svg(&tokens, SvgConfig::default())
        } else {
            render_as_html(&tokens)
        };
        println!("{}", out);
    } else {
        println!("{}", HELP_TEXT);
    }
}
