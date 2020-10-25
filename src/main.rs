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
    ToSocketAddrs,
    TcpStream,
};
use toyssl::http::ParsedUrl;

const HTTP_PORT: u32 = 80;

/// Simple command-line HTTP client
fn main() {
    let args: Vec<String> = env::args().collect();

    println!("length: {}", args.len());

    // arguments validation
    if args.len() < 2 {
        eprintln!("Usage: toyssl [-p http://[username:password@]proxy-host:proxy-port] <URL>");
        exit(1);
    }

    // parsing url
    let parsed_url = ParsedUrl::new(&args[1]);

    if parsed_url.is_none() {
        eprintln!("Error - malformed URL '{}'", args[1]);
        exit(1);
    }

    let parsed_url = parsed_url.unwrap();

    println!("Connecting to host {}", parsed_url.host);

    // resolve ip from hostname
    let addrs = format!("{}:{}", parsed_url.host, HTTP_PORT).to_socket_addrs();

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
                println!("Retrieving document: '{}'", parsed_url.path);
                let mut reader = BufReader::new(&stream);
                let mut writer = BufWriter::new(&stream);
                
                // format HTTP request
                let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", parsed_url.path, parsed_url.host);
                println!("GET request sending...");
                println!("-- Request --\n{}", header);

                tcp_write(&mut writer, &header);
                tcp_read(&mut reader);
            }
        }
    } else {
        eprintln!("Invalid Host:Post number.");
        exit(1);
    }

    exit(0);
}

fn tcp_read(reader: &mut BufReader<&TcpStream>) {
    let mut msg = String::new();
    // reader.read_line(&mut msg).expect("Failed to read lines from tcp stream");
    reader.read_to_string(&mut msg).expect("Failed to read lines from tcp stream");
    println!("{}", msg);
}

fn tcp_write(writer: &mut BufWriter<&TcpStream>, msg: &str) {
    writer.write(msg.as_bytes()).expect("Failed to send message to tcp stream");
    writer.flush().unwrap();
}

