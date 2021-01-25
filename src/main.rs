use linemd::{parse, render_as_html};
use std::io::{prelude::*, stdin};

const HELP_TEXT: &str = include_str!("help.txt");

fn main() {
    if let Some(arg) = std::env::args().nth(1) {
        let md = if arg == "-h" || arg == "--help" {
            println!("{}", HELP_TEXT);
            return;
        } else if arg == "-" {
            let mut input = String::new();
            if let Err(err) = stdin().read_to_string(&mut input) {
                eprintln!("failed to read from stdin: {}", err);
                std::process::exit(1);
            }
            input
        } else {
            match std::fs::read_to_string(&arg) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("failed to read file '{}': {}", arg, err);
                    std::process::exit(1);
                }
            }
        };
        process(md);
    } else {
        println!("{}", HELP_TEXT);
    }
}

fn process(md: String) {
    if !md.is_empty() {
        let html = render_as_html(parse(md));
        println!("{}", html);
    } else {
        println!("no input given, doing nothing");
    }
}
