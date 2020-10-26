extern crate toyssl;

use std::env;
use anyhow::Result;

use toyssl::app::{
    App,
    Client,
};

/// Simple command-line HTTP client & server
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut client = Client::new(args)?;
    client.run()?;

    Ok(())
}

