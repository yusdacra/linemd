const MD: &str = include_str!("all.md");

fn main() {
    let parsed = linemd::parse(MD);
    println!("{:#?}", parsed);
    let html: String = linemd::render_as_html(parsed);
    println!("{}", html);
}
