use bittorrent_rustico::run;
fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
    }
}
