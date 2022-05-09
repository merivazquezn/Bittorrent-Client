use bittorrent_rustico::config::parser;
fn main() {
    parser::parse_from_path("HOLA!");
    println!("Hello, world!");
}
