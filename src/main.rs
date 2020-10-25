extern crate toyssl;

use std::{
    env,
    process::exit,
    io::{
        prelude::*,
        BufReader,
        BufWriter,
    },
};
use std::net::{
    SocketAddr,
    ToSocketAddrs,
    TcpStream,
};
use anyhow::Result;

use toyssl::http::{
    ParsedUrl,
    ParsedProxyUrl,
    http_get,
};

const HTTP_PORT: u32 = 80;

/// Simple command-line HTTP client
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    println!("length: {}", args.len());

    // arguments validation
    if args.len() < 2 {
        eprintln!("Usage: toyssl [-p http://[username:password@]proxy-host:proxy-port] <URL>");
        exit(1);
    }

    // parsing url
    let mut idx: usize = 1;
    let mut parsed_proxy_url: Option<ParsedProxyUrl> = None;
    if args[idx] == "-p" {
        idx += 1;
        parsed_proxy_url = Some(ParsedProxyUrl::new(&args[idx])?);
        idx += 1;
    }
    
    let parsed_url = ParsedUrl::new(&args[idx]);

    if parsed_url.is_none() {
        eprintln!("Error - malformed URL '{}'", args[idx]);
        exit(1);
    }

    let parsed_url = parsed_url.unwrap();

    println!("Connecting to host {}", parsed_url.host);

    // resolve ip from hostname
    let addrs = if let Some(proxy) = &parsed_proxy_url {
        format!("{}:{}", proxy.host, proxy.port).to_socket_addrs()
    } else {
        format!("{}:{}", parsed_url.host, HTTP_PORT).to_socket_addrs()
    };
    
    println!("Resolved IP: {:?}", addrs);

    if addrs.is_err() {
        eprintln!("Error in name resolution.");
        exit(3);
    }

    let mut addrs = addrs.unwrap();

    if let Some(addr) = addrs.find(|x| (*x).is_ipv4()) {
        match TcpStream::connect(addr) {
            Err(_) => {
                eprintln!("Unable to connect to host.");
                exit(4);
            },
            Ok(stream) => {
                http_get(&stream, parsed_url, parsed_proxy_url);
            }
        }
    } else {
        eprintln!("Invalid Host:Post number.");
        exit(1);
    }

    Ok(())
}

