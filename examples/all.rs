const MD: &str = include_str!("all.md");

fn main() {
    let tokens = linemd::parse(MD);
    println!("{:#?}", tokens);
    let html = linemd::render_as_html(tokens.clone());
    println!("{}", html);
    let svg = linemd::render_as_svg(tokens, false);
    println!("{}", svg);
}
