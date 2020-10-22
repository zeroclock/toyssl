extern crate toyssl;

use std::{
    env,
    process::exit,
};
use toyssl::util::ParsedUrl;

const HTTP_PORT: u32 = 80;

/// Simple command-line HTTP client
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: toyssl <URL>");
        exit(1);
    }

    let parsed_url = ParsedUrl::new(&args[1]);

    if parsed_url.is_none() {
        eprintln!("Error - malformed URL '{}'", args[1]);
        exit(1);
    }

    let parsed_url = parsed_url.unwrap();

    println!("Connecting to host {}", parsed_url.host);

    exit(0);
}
