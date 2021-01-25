const MD: &str = include_str!("all.md");

fn main() {
    let parsed = linemd::parse(MD);
    println!("{:#?}", parsed);
}
