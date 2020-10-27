use std::net::{
    ToSocketAddrs,
    TcpStream,
    TcpListener,
};
use anyhow::{
    Result,
    anyhow,
    Context as _,
};
use std::{
    io::{
        BufReader,
        BufWriter,
    },
};

use super::http::{
    ParsedUrl,
    ParsedProxyUrl,
    http_get,
    tcp_write,
    tcp_read_line,
};

const HTTP_PORT: u32 = 80;

pub trait App {
    fn run(&mut self) -> Result<()>;
}

pub struct Client {
    parsed_url: ParsedUrl,
    parsed_proxy_url: Option<ParsedProxyUrl>,
}

impl Client {
    pub fn new(args: &Vec<String>) -> Result<Self> {
        // parsing url
        let mut idx: usize = 2;
        let mut parsed_proxy_url: Option<ParsedProxyUrl> = None;
        if args[idx] == "-p" {
            idx += 1;
            parsed_proxy_url = Some(ParsedProxyUrl::new(&args[idx])?);
            idx += 1;
        }
        
        let parsed_url = ParsedUrl::new(&args[idx]);

        if parsed_url.is_none() {
            return Err(anyhow!("Error - malformed URL '{}'", args[idx]));
        }

        let parsed_url = parsed_url.unwrap();

        Ok(Self {
            parsed_url,
            parsed_proxy_url,
        })
    }
}

impl App for Client {
    fn run(&mut self) -> Result<()> {
        println!("Connecting to host {}", self.parsed_url.host);

        // resolve ip from hostname
        let addrs = if let Some(proxy) = &self.parsed_proxy_url {
            format!("{}:{}", proxy.host, proxy.port).to_socket_addrs()
        } else {
            format!("{}:{}", self.parsed_url.host, HTTP_PORT).to_socket_addrs()
        };
        
        println!("Resolved IP: {:?}", addrs);

        if addrs.is_err() {
            return Err(anyhow!("Error in name resolution."));
        }

        let mut addrs = addrs.unwrap();

        if let Some(addr) = addrs.find(|x| (*x).is_ipv4()) {
            let stream = TcpStream::connect(addr).with_context(|| "Unable to connect to host.")?;
            http_get(&stream, &self.parsed_url, &self.parsed_proxy_url);
        } else {
            return Err(anyhow!("Invalid Host:Port combination."));
        }

        Ok(())
    }
}

enum ResponseStatus {
    NotImplemented,
    Success,
}

use ResponseStatus::*;

pub struct Server;

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    fn process_http_request(&self, stream: &TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut writer = BufWriter::new(stream);
        
        let request = tcp_read_line(&mut reader);
        let mut response = String::new();
        // println!("Received request: ");
        // println!("{}", request);

        let get_pos = request.find("GET");
        if let Some(_) = get_pos {
            response = self.build_response(Success);
        } else {
            response = self.build_response(NotImplemented);
        }

        tcp_write(&mut writer, &response);
        
        Ok(())
    }

    fn build_response(&self, status: ResponseStatus) -> String {
        let mut result = String::new();
        match status {
            NotImplemented => {
                result = format!("HTTP/1.1 501 Error Occurred\r\n\r\n");
            },
            Success => {
                result = format!("HTTP/1.1 200 Success\r\nConnection: Close\r\nContent-Type:text/html\r\n\r\n<html><head><title>Test Page</title></head><body>Nothing here</body></html>\r\n");
            },
        }
        result
    }
}

impl App for Server {
    fn run(&mut self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        for stream in listener.incoming() {
            // TODO: ideally, this would spawn a new thread.
            self.process_http_request(&stream?)?
        }
        Ok(())
    }
}

