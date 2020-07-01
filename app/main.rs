fn main() {
    let argv: Vec<String> = std::env::args().collect();

    println!("{} {}", argv[0], argv[1]);
}
