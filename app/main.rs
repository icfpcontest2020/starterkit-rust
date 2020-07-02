use std::env;
use std::io::Error;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let response = isahc::get(format!("{}?player_key={}", server_url, player_key))?;
    assert!(response.status().is_success());

    Ok(())
}