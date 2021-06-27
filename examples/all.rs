use linemd::Parser;

const MD: &str = include_str!("all.md");

fn main() {
    let tokens = MD.parse_md();
    println!("{:#?}", tokens);
    let html = linemd::render_as_html(&tokens);
    println!("{}", html);
    let svg = linemd::render_as_svg(&tokens, linemd::SvgConfig::default());
    println!("{}", svg);
}
